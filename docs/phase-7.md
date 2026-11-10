# Fase 7: tamaño, firma y distribución

| Qué | Resultado |
|---|---|
| **Tamaño** | Perfil de release mixto (`Cargo.toml`): dependencias con `opt-level = "s"`, salvo las que dibujan, descomprimen y hacen OCR (hayro, vello, pdfcraft-render, pdfcraft-filters, zlib-rs, zune-jpeg, rten, ocrs, epaint), que siguen en `opt-level 3`. Código: 39,8 → 31,4 MB; librería comprimida: 30,7 → 25,9 MB (unos 5 MB menos de APK) |
| **Lo que no se reduce** | Unos 22 MB de datos que son funcionalidad: fuentes de la interfaz, 35 idiomas, CMaps de PDF; y 12 MB de modelos de OCR |
| **Firma** | Clave de release propia, fuera del repositorio; el workflow la lee de los secretos (`docs/releasing.md`) |
| **Releases** | `.github/workflows/release.yml`: tag `v*` o lanzamiento manual → APK firmado, `SHA256SUMS.txt` y notas de `docs/release-notes/` |
| **F-Droid y Obtainium** | Metadatos fastlane (`fastlane/metadata/android`), compilación sin firmar (`-Pdexpdfcraft.unsigned=true`) y borrador de receta (`docs/fdroid.md`) |

## Pendiente

- Medir en el dispositivo el tiempo de arranque y de render con el perfil mixto. Si algo va más
  lento, se pasa esa crate a `opt-level 3`.
- Pedir la inclusión en F-Droid, cuando se quiera (`docs/fdroid.md`).
- El OCR en otros idiomas con Tesseract, aparcado (`docs/phase-5.md`).
