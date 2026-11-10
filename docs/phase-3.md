# Fase 3: archivos, portapapeles y «Abrir con»

PdfCraft trabaja con rutas de archivo y Android con documentos del Storage Access Framework (SAF).
Esta fase los une sin cambiar la interfaz de PdfCraft: todo pasa por los shims de `rfd` y `arboard`
del fork y por el canal de eventos del sistema que ya usa la app de escritorio.

## Cómo funciona

```
PdfCraft (Rust) ──rfd / arboard──▶ shims ──backends──▶ android/{pickers,clipboard}.rs
                                                           │ JNI: MainActivity.bridge(op, json, bytes)
                                                           ▼
                                   MainActivity.kt ── DocumentStore.kt (SAF ↔ copias locales)
                                        │            └ ClipboardBridge.kt (ClipboardManager)
                                        └── nativeEvent("picked" | "open", json) ──▶ Rust
```

- **El puente JNI es mínimo.** Hay un método Kotlin, `bridge(op, json, bytes)`, y dos funciones
  nativas, `nativeRegister` y `nativeEvent`. Todo viaja como JSON y bytes (`src/protocol.rs`, con
  tests). Solo `android_main` y esas dos funciones exportadas llevan `unsafe` (el atributo
  `no_mangle`). La JVM es la de android-activity (`JavaVM::singleton`).
- **Abrir.** `ACTION_OPEN_DOCUMENT` muestra el selector del sistema. El documento se copia a
  `files/documents/<id>/<nombre>` y PdfCraft recibe esa ruta. El `<id>` sale del URI, así que el
  mismo documento siempre se abre en la misma ruta, y las pestañas y los recientes no se duplican.
- **Guardar.** Un `FileObserver` vigila cada carpeta de copias. Cuando PdfCraft guarda (escribe
  `.<nombre>…tmp` y lo renombra), el archivo se copia al documento original. Si el documento es de
  solo lectura o falla la escritura, sale un aviso que sugiere «Guardar como».
- **Guardar como** (`ACTION_CREATE_DOCUMENT`) y **carpetas de salida** (`ACTION_OPEN_DOCUMENT_TREE`)
  funcionan igual. En una carpeta, cada archivo nuevo que escribe PdfCraft se crea en la carpeta elegida.
- **Permisos.** Se pide el permiso persistente de cada documento, y la relación entre ruta y
  documento se guarda. Así guardar sigue funcionando después de reiniciar, también desde «Recientes».
- **Portapapeles.** Texto, HTML con su texto plano, imágenes (PNG a través del `FileProvider` de la
  app) y listas de archivos.
- **«Abrir con» y «Compartir».** Hay intent filters para `VIEW`/`EDIT` y `SEND`/`SEND_MULTIPLE` de
  PDF. Los documentos llegan a PdfCraft como `OsEvent::Open`, igual que un doble clic en el Finder.
- **Robustez.** Si Android recrea la actividad, los selectores que estaban abiertos se cancelan, para
  que PdfCraft no se quede esperando.

## Verificado aquí

- La librería compila y pasa clippy sin avisos para Android arm64 y para el host. Los tests del
  protocolo y del idioma pasan: 5 tests.
- El APK de depuración (arm64, 95 MB) se construye y el Kotlin compila sin avisos. El APK exporta
  `android_main`, `…MainActivity_nativeRegister` y `…MainActivity_nativeEvent`, y su manifiesto
  incluye los intent filters de PDF (`VIEW`, `EDIT`, `SEND`, `SEND_MULTIPLE`) y el `FileProvider`.
- **Nada de esto se ha ejecutado en un dispositivo.** No hay emulador ni hardware Android en este
  entorno; la lista de abajo es lo que hay que probar.

## Por probar en un dispositivo

1. Archivo ▸ Abrir: se ve el selector del sistema y el PDF se abre en una pestaña.
2. Editar y guardar (Ctrl+S): el cambio aparece en el archivo original, por ejemplo en «Mis archivos».
3. Guardar como en otra carpeta o en Google Drive.
4. Exportar a una carpeta (Exportar ▸ imágenes, por ejemplo): los archivos aparecen en la carpeta elegida.
5. «Abrir con» desde «Mis archivos» y «Compartir» desde otra app, con la app abierta y con la app cerrada.
6. Copiar texto e imágenes de la página y pegarlos en otra app, y al revés.
7. Reiniciar la app, abrir un documento de Recientes, editarlo y guardar: el cambio llega al original.

## Correcciones tras la primera prueba en DeX

| Síntoma | Causa | Arreglo |
|---|---|---|
| La rueda del ratón no hace nada; tampoco hay hover ni clic derecho | El backend Android de winit convertía cualquier evento de puntero en un toque e ignoraba `ACTION_SCROLL`, el hover y los botones | `vendor/winit` (el winit de storytold más el parche W-DEX1, en `vendor/winit/PATCHES.md`): un ratón genera `CursorMoved`, `MouseInput` según `button_state` (izquierdo, derecho, central, atrás, adelante) y `MouseWheel` según `AXIS_VSCROLL`/`AXIS_HSCROLL`. Lo protege el test `android_mice_are_mice_not_fingers` |
| Al cerrar no pregunta si se quieren guardar los cambios | winit ignora la destrucción de la actividad, así que PdfCraft nunca recibía la petición de cierre | «Atrás» (gesto o tecla) llega a PdfCraft como cierre de ventana (`shell.rs`) y sale su pregunta «¿guardar cambios?». Solo cuando PdfCraft deja cerrar, la app pide a Android que termine la actividad. La X de una ventana de DeX la gestiona el sistema y no se puede interceptar; para ese caso, PdfCraft guarda en su recuperación cada vez que la ventana pierde el foco, y el siguiente arranque ofrece recuperar el trabajo |
| Tras cerrar, al volver a abrir la app se cierra sola | El proceso seguía vivo con el bucle de eventos de winit, que no se puede crear dos veces en un proceso | Al destruirse la actividad se termina el proceso (`onDestroy`), después de esperar a que acaben de escribirse los guardados pendientes en los documentos originales. Cada arranque es un proceso nuevo |

### Segunda ronda (tras probar el APK anterior)

| Síntoma | Causa | Arreglo |
|---|---|---|
| La rueda seguía sin funcionar | GameActivity solo copia los ejes X e Y de los eventos; `AXIS_VSCROLL`/`AXIS_HSCROLL` llegaban a cero | `android_main` activa los dos ejes (`enable_motion_axis`) |
| El clic izquierdo dejó de funcionar | Probablemente el ratón de DeX no rellena los bits de botón en `DOWN`/`UP`, y W-DEX1 dependía solo de ellos | Si faltan los bits, `DOWN`/`UP` cuentan como botón principal y `BUTTON_PRESS`/`BUTTON_RELEASE` usan su botón. Los primeros 40 clics y giros de rueda se escriben en el log (`adb logcat -s dexpdfcraft`) para confirmarlo en el dispositivo |
| Tras cerrar, la app se cierra al volver a abrirla | `GameActivity.onDestroy` espera en el hilo principal a que termine `android_main`, pero el bucle de winit ignoraba la destrucción: la actividad se quedaba colgada y el proceso, inutilizable | W-DEX2: `MainEvent::Destroy` termina el bucle. Si aun así Android reutiliza un proceso que ya ejecutó la app, `RestartActivity` (en el proceso `:restart`) la vuelve a abrir en uno nuevo |
| Sin `adb` no hay forma de saber qué falló | — | Un error de Rust (panic) o de Java se guarda en `crash.txt` y se muestra en el siguiente arranque, con un botón para copiarlo |

### Tercera ronda

En DeX la rueda funcionaba, pero ninguno de los dos botones; en el móvil con el dedo todo iba bien.

- **El ratón ya no pasa por GameActivity ni por winit.** `MainActivity` pone sus propios listeners en la
  superficie de GameActivity. El ratón, el touchpad o una herramienta de tipo ratón van con
  `nativePointer` a `src/pointer.rs`, que los convierte en eventos de egui: movimiento, hover,
  botones (con la API de `MotionEvent`: `getButtonState`, `getActionButton`, y `DOWN`/`UP` cuando los
  bits vienen vacíos), rueda y modificadores. El táctil sigue yendo a GameActivity
  (`processMotionEvent`), que es lo que ya funcionaba. Tests en `pointer.rs`.
- **Diagnóstico sin adb.** `files/diag.log` registra el ciclo de vida de la actividad, la pantalla, la
  densidad, los primeros 60 eventos de ratón tal como llegan de Android y los hitos de la librería
  (arranque, renderizador, fin del bucle, panics). Si la sesión anterior no terminó con normalidad o
  hubo un error, el siguiente arranque muestra el informe con un botón «Copiar informe». La build de
  depuración añade un segundo icono, «Diagnóstico DexPdfCraft», para verlo cuando se quiera.

### Cuarta ronda (con el informe de diagnóstico del dispositivo)

El informe mostró dos cosas: el ratón ya funcionaba, y los arranques fallidos terminaban el bucle de
eventos en el mismo milisegundo en que DeX cambiaba el tamaño de la ventana, antes de que existiera.

| Síntoma | Causa | Arreglo |
|---|---|---|
| A veces, al volver a abrirla, la app se cierra sola | Android avisa del cambio de tamaño (DeX amplía la ventana al abrirla) antes de crear la ventana nativa. winit lo envía como `ScaleFactorChanged` y eframe, que aún no tiene ventana, lo toma como el final del programa (`EventResult::Exit`) | W-DEX3 en `vendor/winit`: no se envía ningún evento de ventana antes del primer `Resumed`. Lo protege el test `android_window_events_wait_for_the_window` |
| Del teclado solo funcionan las teclas de función, las flechas y Re Pág/Av Pág | winit en Android traduce por tabla las teclas que conoce, pero no obtiene el carácter de letras, números y símbolos | `MainActivity` lee los teclados físicos (`KeyEvent`: carácter según la distribución, con Mayús, AltGr y teclas muertas como ´ + a = á) y los pasa con `nativeKey` a `src/keyboard.rs`. Ahí se convierten en eventos de egui: tecla lógica según la distribución, tecla física por posición, texto y `ModifiersChanged`. Tests en `keyboard.rs`. El teclado en pantalla sigue yendo por GameActivity |

## Pendiente

- **Imprimir** con `PrintManager` y **compartir** desde PdfCraft. La app de escritorio imprime con
  la cola del sistema (craft-print), que no existe en Android. Hay que llevar el PDF que PdfCraft
  compone a un `PrintDocumentAdapter`.
- Un documento abierto desde «Recientes» usa la copia local, que puede estar desactualizada si el
  original cambió fuera de la app. Habrá que refrescar la copia al abrir desde Recientes.
- Leer en voz alta (TTS) y arrastrar y soltar desde «Mis archivos» en DeX.
