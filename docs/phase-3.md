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

## Pendiente

- **Imprimir** con `PrintManager` y **compartir** desde PdfCraft. La app de escritorio imprime con
  la cola del sistema (craft-print), que no existe en Android. Hay que llevar el PDF que PdfCraft
  compone a un `PrintDocumentAdapter`.
- Un documento abierto desde «Recientes» usa la copia local, que puede estar desactualizada si el
  original cambió fuera de la app. Habrá que refrescar la copia al abrir desde Recientes.
- Leer en voz alta (TTS) y arrastrar y soltar desde «Mis archivos» en DeX.
