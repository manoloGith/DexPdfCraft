import javax.inject.Inject
import org.gradle.process.ExecOperations

plugins {
    alias(libs.plugins.android.application)
}

// The Rust half of the app: this Gradle project is the android/ folder of the `dexpdfcraft` crate
// (apps/dexpdfcraft), in the cargo workspace at the root of the repository.
val cargoWorkspaceDir: File = rootDir.parentFile.parentFile.parentFile
val rustAbis: List<String> =
    providers.gradleProperty("dexpdfcraft.abis").getOrElse("arm64-v8a,x86_64").split(',').map { it.trim() }.filter { it.isNotEmpty() }
// Android 10: the first release with desktop mode on external displays; DeX itself runs on 9+.
val minApi = 29
val libName = "dexpdfcraft"

/** rustup's `cargo` (Android Studio may not see rustup's PATH), else `cargo` from PATH; `CARGO` overrides. */
val cargoExecutable: String = run {
    val exe = if (System.getProperty("os.name").lowercase().startsWith("windows")) "cargo.exe" else "cargo"
    val rustup = File(System.getProperty("user.home"), ".cargo/bin/$exe")
    providers.environmentVariable("CARGO").orNull ?: if (rustup.isFile) rustup.absolutePath else exe
}

/** Builds `libdexpdfcraft.so` for each ABI with cargo-ndk (release profile unless [release] is false). */
abstract class CargoNdkBuild : DefaultTask() {
    @get:Inject abstract val execOperations: ExecOperations
    @get:Input abstract val cargo: Property<String>
    @get:Input abstract val workspaceDir: Property<String>
    @get:Input abstract val ndkDir: Property<String>
    @get:Input abstract val abis: ListProperty<String>
    @get:Input abstract val apiLevel: Property<Int>
    @get:Input abstract val release: Property<Boolean>
    @get:Input abstract val library: Property<String>
    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @TaskAction
    fun build() {
        val targets = abis.get().flatMap { listOf("-t", it) }
        val out = outputDir.get().asFile.absolutePath
        val cargoHome = System.getenv("CARGO_HOME")?.takeIf { it.isNotBlank() } ?: File(System.getProperty("user.home"), ".cargo").absolutePath
        // Reproducible: the library mustn't hold the paths it was built in.
        val remap = listOf(
            "--remap-path-prefix=${File(workspaceDir.get()).absolutePath}=/dexpdfcraft",
            "--remap-path-prefix=$cargoHome=/cargo",
        )
        val profile = if (release.get()) listOf("--release") else emptyList()
        // The interface translations (scripts/fetch-build-inputs.sh); without them, English only.
        val i18n = System.getenv("CRAFT_I18N_DIR")?.takeIf { it.isNotBlank() }
            ?: File(workspaceDir.get(), ".build-inputs/craft-i18n").takeIf { it.isDirectory }?.absolutePath
        execOperations.exec {
            workingDir(workspaceDir.get())
            environment("ANDROID_NDK_HOME", ndkDir.get())
            if (i18n != null) environment("CRAFT_I18N_DIR", i18n)
            environment("CARGO_ENCODED_RUSTFLAGS", remap.joinToString("\u001f"))
            commandLine(
                listOf(cargo.get(), "ndk") + targets +
                    listOf("--platform", apiLevel.get().toString(), "-o", out, "build") + profile +
                    listOf("--locked", "-p", library.get(), "--lib"),
            )
        }
        // cargo-ndk copies every shared library cargo left behind; only the app's ships.
        File(out).walk().filter { it.isFile && it.name != "lib${library.get()}.so" }.forEach { it.delete() }
    }
}

/**
 * The text-recognition models (scripts/fetch-build-inputs.sh puts them in .build-inputs/models,
 * or DEXPDFCRAFT_MODELS_DIR names them) copied into the APK's assets/models. Without them the app
 * still builds; Scan & OCR then says they aren't installed.
 */
abstract class OcrModels : DefaultTask() {
    @get:Input abstract val modelsDir: Property<String>
    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @TaskAction
    fun copy() {
        val out = File(outputDir.get().asFile, "models")
        out.deleteRecursively()
        val files = File(modelsDir.get()).listFiles { f -> f.isFile && !f.name.endsWith(".part") }.orEmpty()
        if (files.none { it.extension == "onnx" || it.extension == "rten" }) {
            logger.warn("OCR models not found in ${modelsDir.get()}: run scripts/fetch-build-inputs.sh")
            return
        }
        out.mkdirs()
        for (f in files) f.copyTo(File(out, f.name), overwrite = true)
    }
}

/**
 * Tesseract, PdfCraft's second OCR engine (scripts/build-tesseract-android.sh): the program as
 * lib/<abi>/libtesseract.so and its language data as assets/tessdata, when they have been built.
 * Without them the app builds; Scan & OCR then offers only the built-in engine (English).
 */
abstract class TesseractFiles : DefaultTask() {
    @get:Input abstract val buildDir: Property<String>
    @get:Input abstract val abis: ListProperty<String>
    @get:Input abstract val assets: Property<Boolean>
    @get:OutputDirectory abstract val outputDir: DirectoryProperty

    @TaskAction
    fun copy() {
        val out = outputDir.get().asFile
        out.deleteRecursively()
        out.mkdirs()
        val root = File(buildDir.get())
        if (assets.get()) {
            val data = File(root, "tessdata").listFiles { f -> f.isFile && (f.extension == "traineddata" || f.name == "LICENSE") }.orEmpty()
            if (data.isNotEmpty()) {
                val dir = File(out, "tessdata").apply { mkdirs() }
                data.forEach { it.copyTo(File(dir, it.name), overwrite = true) }
            }
        } else {
            for (abi in abis.get()) {
                val program = File(root, "tesseract/$abi/libtesseract.so")
                if (program.isFile) program.copyTo(File(out, "$abi/libtesseract.so"), overwrite = true)
                else logger.warn("Tesseract not built for $abi: run scripts/build-tesseract-android.sh $abi")
            }
        }
    }
}

android {
    namespace = "io.github.dexpdfcraft"
    compileSdk = 36
    ndkVersion = "29.0.14206865"

    defaultConfig {
        applicationId = "io.github.dexpdfcraft"
        minSdk = minApi
        targetSdk = 36
        versionCode = providers.gradleProperty("dexpdfcraft.versionCode").get().toInt()
        versionName = providers.gradleProperty("dexpdfcraft.version").get()
        ndk { abiFilters.addAll(rustAbis) }
    }

    // The release key, when the build is given one (the release workflow passes it from the
    // repository's secrets). Without one a release build is signed with the debug key: it installs,
    // but a later release signed with the real key can't update it.
    val releaseKeystore: String? = providers.environmentVariable("DEXPDFCRAFT_KEYSTORE").orNull?.takeIf { it.isNotBlank() }
    signingConfigs {
        if (releaseKeystore != null) {
            create("release") {
                storeFile = file(releaseKeystore)
                storePassword = providers.environmentVariable("DEXPDFCRAFT_KEYSTORE_PASSWORD").orNull
                keyAlias = providers.environmentVariable("DEXPDFCRAFT_KEY_ALIAS").orNull
                keyPassword = providers.environmentVariable("DEXPDFCRAFT_KEY_PASSWORD").orNull
            }
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
        }
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // -Pdexpdfcraft.unsigned=true (F-Droid's build): not signed at all, so the store signs it,
            // or checks it against the published APK.
            signingConfig = if (providers.gradleProperty("dexpdfcraft.unsigned").orNull == "true") {
                null
            } else {
                signingConfigs.findByName("release") ?: signingConfigs.getByName("debug")
            }
            vcsInfo { include = false }
        }
    }
    dependenciesInfo {
        includeInApk = false
        includeInBundle = false
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    androidResources {
        // The OCR models are read by the library; storing them uncompressed lets them be
        // copied out quickly (they barely compress anyway).
        noCompress += listOf("onnx", "rten", "traineddata")
    }
    packaging {
        // Native libraries are extracted to the app's native-library folder when installed:
        // Tesseract (libtesseract.so) is a program PdfCraft runs, and Android runs programs
        // only from there.
        jniLibs { useLegacyPackaging = true }
    }
}

androidComponents {
    onVariants { variant ->
        val name = variant.name.replaceFirstChar { it.uppercase() }
        val cargoBuild = tasks.register<CargoNdkBuild>("cargoBuild$name") {
            group = "rust"
            description = "Builds lib$libName.so for $rustAbis"
            cargo.set(cargoExecutable)
            workspaceDir.set(cargoWorkspaceDir.absolutePath)
            ndkDir.set(androidComponents.sdkComponents.ndkDirectory.map { it.asFile.absolutePath })
            abis.set(rustAbis)
            apiLevel.set(minApi)
            // a debug APK still gets an optimised library: the renderer is unusable at opt-level 1
            release.set(providers.gradleProperty("dexpdfcraft.rustDebug").orNull != "true")
            library.set(libName)
            // cargo knows what changed
            outputs.upToDateWhen { false }
        }
        variant.sources.jniLibs?.addGeneratedSourceDirectory(cargoBuild, CargoNdkBuild::outputDir)
        val models = tasks.register<OcrModels>("ocrModels$name") {
            modelsDir.set(
                providers.environmentVariable("DEXPDFCRAFT_MODELS_DIR").orElse(File(cargoWorkspaceDir, ".build-inputs/models").absolutePath),
            )
            outputs.upToDateWhen { false }
        }
        variant.sources.assets?.addGeneratedSourceDirectory(models, OcrModels::outputDir)
        val inputs = File(cargoWorkspaceDir, ".build-inputs").absolutePath
        val tesseract = tasks.register<TesseractFiles>("tesseract$name") {
            buildDir.set(inputs)
            abis.set(rustAbis)
            assets.set(false)
            outputs.upToDateWhen { false }
        }
        variant.sources.jniLibs?.addGeneratedSourceDirectory(tesseract, TesseractFiles::outputDir)
        val tessdata = tasks.register<TesseractFiles>("tessdata$name") {
            buildDir.set(inputs)
            abis.set(rustAbis)
            assets.set(true)
            outputs.upToDateWhen { false }
        }
        variant.sources.assets?.addGeneratedSourceDirectory(tessdata, TesseractFiles::outputDir)
    }
}

tasks.configureEach {
    if (name.contains("ArtProfile")) enabled = false
}

dependencies {
    // GameActivity's Java side. Its prefab C++ glue stays off: android-activity compiles its own.
    implementation(libs.games.activity)
    implementation(libs.androidx.appcompat)
}
