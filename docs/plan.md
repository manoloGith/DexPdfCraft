# Plan por fases

Enfoque: **compilar para Android la interfaz egui de escritorio de PdfCraft** (`pdfcraft-ui-egui`,
~105.000 líneas) en lugar de reescribirla, y añadir solo la capa de plataforma Android. Así se
conserva la disposición del escritorio y todas las funciones, y se sigue a upstream moviendo un
submódulo.

| Fase | Contenido | Estado |
|---|---|---|
| 0 | Repositorio: submódulo `upstream/`, workspace Cargo con los `[patch]` de upstream, Gradle + cargo-ndk, CI, licencias y marcas | ✅ |
| 1 | Prueba técnica: eframe/winit en Android (GameActivity), wgpu con glow de reserva, motor y renderizador de PdfCraft en ARM64, entrada de DeX | ✅ código · ⬜ probar en DeX ([phase-1.md](phase-1.md)) |
| 2 | `pdfcraft-ui-egui` en Android: shims de `rfd` y `arboard` en el fork ([fork.md](fork.md)), sin logotipos de ArtCraft; la app arranca `PdfCraftApp` con las carpetas de datos en el almacenamiento privado; compila para arm64 | ✅ código ([phase-2.md](phase-2.md)): compila para arm64, arranca en escritorio, idioma del sistema, `/system/fonts` · ⬜ probar en DeX |
| 3 | Puente Kotlin ↔ Rust ([phase-3.md](phase-3.md)): SAF abrir/guardar/guardar como/carpetas con escritura de vuelta, portapapeles, «Abrir con» y «Compartir hacia» la app | ✅ código · ⬜ probar en DeX · ⬜ imprimir (`PrintManager`), compartir desde la app, TTS, arrastrar y soltar |
| 4 | DeX y escritorio: detectar el modo escritorio, escala de interfaz, cursores (`PointerIcon`), atajos completos, sin decoraciones propias | ⬜ |
| 5 | Paridad de funciones con las listas de upstream (`upstream/docs/*-parity.md`): impresión, OCR en proceso, firma, formularios JS, redacción, exportación | ⬜ |
| 6 | Opcional: móvil y tablet sin DeX (misma interfaz, escala mayor, gestos) | ⬜ |
| 7 | Rendimiento y memoria en ARM, autoguardado ante cierre en segundo plano, firma del APK, F-Droid | ⬜ |

## Riesgos conocidos

1. **Entrada de texto**: egui en Android tiene soporte limitado de teclado en pantalla e IME. En DeX
   (teclado físico) basta; la composición CJK necesitará trabajo.
2. **Forks de egui/winit de storytold**: conservan el soporte de Android de winit
   (`android-native-activity`); si un parche lo rompe, habrá que mantener un parche propio aquí.
3. **Drivers GPU** (Mali, Adreno) con wgpu: mitigado con glow.
4. **Seguir a upstream**: cambia a diario. `scripts/sync-upstream.sh` rebasa la rama del fork
   sobre el `main` de upstream, comprueba que los `[patch.crates-io]` siguen iguales y vuelve a tomar
   su `Cargo.lock` ([fork.md](fork.md)).
