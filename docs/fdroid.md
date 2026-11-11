# Distribución: GitHub, Obtainium y F-Droid

## GitHub y Obtainium (ya disponible)

Cada versión se publica como release de GitHub con el APK firmado (`docs/releasing.md`).
[Obtainium](https://github.com/ImranR98/Obtainium) puede seguir el repositorio
`https://github.com/manoloGith/DexPdfCraft` e instalar las actualizaciones según salen.

## F-Droid (preparado, sin enviar)

El repositorio tiene los metadatos en el formato que F-Droid lee
(`fastlane/metadata/android/{en-US,es-ES}`: título, descripciones, notas por `versionCode`, icono y
capturas). Para pedir la inclusión hay que abrir una *merge request* en
[fdroiddata](https://gitlab.com/fdroid/fdroiddata) con un archivo `metadata/io.github.dexpdfcraft.yml`.
Este es el borrador:

```yaml
Categories:
  - Reading
License: MIT OR Apache-2.0
SourceCode: https://github.com/manoloGith/DexPdfCraft
IssueTracker: https://github.com/manoloGith/DexPdfCraft/issues
Changelog: https://github.com/manoloGith/DexPdfCraft/releases

AutoName: DexPdfCraft

RepoType: git
Repo: https://github.com/manoloGith/DexPdfCraft

Builds:
  - versionName: 0.1.0
    versionCode: 1
    commit: v0.1.0
    subdir: apps/dexpdfcraft/android/app
    submodules: true
    sudo:
      - apt-get update
      - apt-get install -y cmake ninja-build
    gradle:
      - yes
    output: build/outputs/apk/release/app-release-unsigned.apk
    prebuild:
      - curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
      - source $HOME/.cargo/env
      - rustup toolchain install $(sed -n 's/^channel = "\(.*\)"/\1/p' ../../../../rust-toolchain.toml)
      - cargo install cargo-ndk --version 4.1.2 --locked
      - ../../../../scripts/fetch-build-inputs.sh
    gradleprops:
      - dexpdfcraft.abis=arm64-v8a
      - dexpdfcraft.unsigned=true
    ndk: 29.0.14206865

AutoUpdateMode: Version
UpdateCheckMode: Tags ^v[0-9.]+$
CurrentVersion: 0.3.0
CurrentVersionCode: 3
```

Puntos que la revisión de F-Droid mirará:

- **Entradas descargadas al compilar.** `fetch-build-inputs.sh` baja las traducciones (craft-i18n,
  en una revisión fija) y los modelos de OCR (comprobados con su SHA-256). F-Droid puede pedir
  declararlos como `srclibs`.
- **Firma.** F-Droid firma con su propia clave, salvo que se active la verificación de builds
  reproducibles contra el APK de GitHub. La biblioteca nativa ya se compila sin rutas locales
  (`--remap-path-prefix`) y el APK no lleva información de dependencias ni del control de versiones.
- **Dependencias no libres:** no hay ninguna. GameActivity y AndroidX son Apache-2.0; no hay
  servicios de Google, analítica ni anuncios.
- **Sin firmar para F-Droid:** `-Pdexpdfcraft.unsigned=true` produce `app-release-unsigned.apk`.
  Sin esa opción ni las variables `DEXPDFCRAFT_KEYSTORE*`, el release se firma con la clave de
  depuración.
