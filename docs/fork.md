# El fork de PdfCraft

DexPdfCraft necesita unos pocos cambios en PdfCraft para que su interfaz compile y se distribuya en
Android. Viven en la rama **`dexpdfcraft/android`** de un fork de `storytold/pdfcraft`:

| Cambio | Por qué |
|---|---|
| `android/rfd-shim`, `android/arboard-shim` y `crates/ui-egui/Cargo.toml` | `rfd` (diálogos) y `arboard` (portapapeles) no tienen backend Android. Los shims toman sus nombres: en los demás sistemas reexportan las crates reales, así que allí no cambia nada; en Android los responde un backend que instala la app (SAF y `ClipboardManager`, fase 3). |
| `crates/ui-egui/src/widgets.rs` | Los logotipos de ArtCraft son marcas que las versiones modificadas deben quitar (`docs/brand/LICENSE-brand.txt`). El fork no incrusta ninguno. |

## Mientras el fork no está en GitHub

Esta sesión no puede crear repositorios en tu cuenta, así que los commits del fork se guardan como
parches en `patches/upstream/`. El submódulo sigue apuntando al commit de upstream y
`scripts/apply-fork-patches.sh` crea encima la rama `dexpdfcraft/android`. El CI lo ejecuta antes de
compilar, y en un clon nuevo también hay que hacerlo:

```bash
git clone --recursive https://github.com/manoloGith/DexPdfCraft
```

```bash
cd DexPdfCraft && scripts/apply-fork-patches.sh
```

Para trabajar en el fork: haz commit en `upstream/` sobre la rama `dexpdfcraft/android` y después
ejecuta `scripts/export-fork-patches.sh`, que regenera `patches/upstream/`.

`.gitmodules` lleva `ignore = all` para que el submódulo con los parches aplicados no aparezca como
modificado. Se quitará cuando el submódulo apunte al fork.

## Pasar al fork en GitHub (una vez)

1. En GitHub, pulsa **Fork** en <https://github.com/storytold/pdfcraft> y crea `manoloGith/pdfcraft`.
2. Añade ese repositorio a las sesiones de Claude Code y pídeme que haga el cambio. Haré esto:
   - subiré la rama `dexpdfcraft/android` al fork;
   - cambiaré la URL del submódulo y lo fijaré a esa rama;
   - quitaré `ignore = all`, `patches/` y el paso del CI.

Para actualizar a una versión nueva de PdfCraft hay que rebasar `dexpdfcraft/android` sobre el
`main` de upstream; `scripts/sync-upstream.sh` comprueba después que los `[patch]` de Cargo siguen
coincidiendo.
