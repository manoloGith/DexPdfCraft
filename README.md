# DexPdfCraft

La interfaz de escritorio de [PdfCraft](https://github.com/storytold/pdfcraft) en Android, pensada
para **Samsung DeX** y para el modo de ventanas de escritorio de Android (monitor, ratón y teclado).

El objetivo es que la app tenga la misma disposición que PdfCraft en escritorio (panel «All tools»,
barra rápida, pestañas, paneles laterales, paleta de comandos) y **todas** sus funciones. Para eso no
se reescribe la interfaz: se compila para Android la propia interfaz egui de PdfCraft
(`pdfcraft-ui-egui`) y se le añade una capa de plataforma Android.

> Estado: **fase 2**. La app ya arranca la interfaz completa de PdfCraft (compila para Android arm64 y
> funciona en escritorio). Faltan el selector de archivos y el portapapeles de Android (fase 3): hasta
> entonces la app abre un PDF de muestra y los diálogos de abrir y guardar se cancelan. Plan en
> [docs/plan.md](docs/plan.md); los cambios a PdfCraft, en [docs/fork.md](docs/fork.md).

## Cómo está hecho

```
DexPdfCraft/
├── upstream/                  submódulo: storytold/pdfcraft, fijado a un commit, sin cambios
├── apps/dexpdfcraft/          el crate Rust de la app (cdylib que carga Android)
│   ├── src/lib.rs             android_main → eframe (wgpu, con glow de reserva)
│   ├── src/pdfcraft.rs        arranca la interfaz de PdfCraft (PdfCraftApp)
│   ├── src/app.rs             ventana de diagnóstico de la fase 1 (feature `probe`)
│   ├── src/doc.rs             abrir y renderizar con pdfcraft-render
│   ├── src/main.rs            la misma ventana en escritorio, para iterar sin dispositivo
│   └── android/               proyecto Gradle: GameActivity + manifiesto para DeX
├── patches/upstream/          commits del fork de PdfCraft (hasta que esté en GitHub)
├── scripts/                   sync-upstream, apply/export-fork-patches
└── docs/                      plan por fases y notas de cada fase
```

- **Motor**: los 31 crates de PdfCraft (`upstream/crates/*`) se usan como dependencias por ruta; no
  hay copia ni fork del motor, así que actualizar es mover el submódulo.
- **Interfaz**: egui 0.37 y winit (los forks fijados de storytold) con el soporte de Android de
  `android-activity` (GameActivity). wgpu dibuja con Vulkan o GLES; si no arranca, glow (OpenGL ES).
- **Kotlin**: solo una `MainActivity` mínima. En la fase 3 se encargará de lo que solo Android ofrece
  (Storage Access Framework, «Abrir con», compartir, imprimir, lectura en voz alta) vía JNI.

## Compilar

Requisitos:

- `rustup` (instala la versión y los targets de `rust-toolchain.toml`)
- `cargo install cargo-ndk --version 4.1.2 --locked`
- Android SDK (plataforma 36, build-tools 36.0.0) y NDK 29.0.14206865
- JDK 17 o superior

Primero clona con el submódulo:

```bash
git clone --recursive https://github.com/manoloGith/DexPdfCraft
```

Después, los cambios del fork de PdfCraft sobre el submódulo ([docs/fork.md](docs/fork.md)):

```bash
cd DexPdfCraft && scripts/apply-fork-patches.sh
```

Después, el APK de depuración (arm64-v8a para el dispositivo, x86_64 para el emulador):

```bash
cd DexPdfCraft/apps/dexpdfcraft/android && ./gradlew assembleDebug
```

Solo arm64, que es más rápido:

```bash
./gradlew assembleDebug -Pdexpdfcraft.abis=arm64-v8a
```

Instalar en un dispositivo conectado:

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

La misma ventana en escritorio Linux, sin dispositivo:

```bash
cargo run -p dexpdfcraft --features desktop -- archivo.pdf
```

El CI (`.github/workflows/android.yml`) compila el APK en cada push y lo deja como artefacto.

## Licencia y marcas

MIT o Apache-2.0, como PdfCraft. Los logotipos de ArtCraft no se usan: ver [NOTICE.md](NOTICE.md).
