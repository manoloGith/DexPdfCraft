# El fork de PdfCraft

`upstream/` es un submódulo de **[manoloGith/pdfcraft](https://github.com/manoloGith/pdfcraft)**, un
fork de `storytold/pdfcraft`, en la rama **`dexpdfcraft/android`**. La rama es el `main` de upstream
más unos pocos commits que DexPdfCraft necesita para que la interfaz compile y se distribuya en
Android:

| Commit | Por qué |
|---|---|
| Shims de `rfd` y `arboard` (`android/`, `crates/ui-egui/Cargo.toml`) | `rfd` (diálogos) y `arboard` (portapapeles) no tienen backend Android. Los shims toman sus nombres: en los demás sistemas reexportan las crates reales, así que allí no cambia nada; en Android los responde un backend que instala la app (SAF y `ClipboardManager`, fase 3). |
| Sin logotipos de ArtCraft (`crates/ui-egui/src/widgets.rs`) | Son marcas que las versiones modificadas deben quitar (`docs/brand/LICENSE-brand.txt`). |
| Fuentes de `/system/fonts` (`platform/system_fonts.rs`) | Android guarda ahí sus fuentes y no tiene fontconfig. |

## Trabajar en el fork

En `upstream/`, `origin` es el fork y `storytold` es el proyecto original:

```bash
git -C upstream checkout dexpdfcraft/android
```

Después de hacer commit en `upstream/`, sube la rama y registra aquí el nuevo commit del submódulo:

```bash
git -C upstream push origin dexpdfcraft/android
```

```bash
git add upstream && git commit -m "upstream: …"
```

Un clon nuevo solo necesita `git clone --recursive` (o `git submodule update --init`).

## Actualizar a una versión nueva de PdfCraft

```bash
scripts/sync-upstream.sh
```

El script rebasa `dexpdfcraft/android` sobre el `main` de storytold, comprueba que los
`[patch.crates-io]` de este workspace siguen coincidiendo con los de upstream y vuelve a tomar su
`Cargo.lock`. Luego hay que compilar y probar, subir la rama con `--force-with-lease` y hacer commit
del submódulo.

Los cambios que puedan servir a todos, como los shims, se pueden proponer a storytold más adelante
como pull request.
