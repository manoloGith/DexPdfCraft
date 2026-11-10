# Fase 5: funciones del escritorio que necesitaban a Android

PdfCraft hace casi todo dentro del motor, que funciona igual en Android: formularios y su
JavaScript, firmas digitales, redacción, exportación, comparación, optimización, organizar páginas.
Esta fase cubre lo que el escritorio resolvía con el sistema operativo.

| Función | En escritorio | En Android |
|---|---|---|
| **Imprimir** | Cola del sistema (CUPS, Windows) | El diálogo Imprimir de PdfCraft ofrece una impresora, «Android», cuyo spooler (`android/print.rs`) guarda el PDF ya compuesto (páginas, escala, folleto, n-up, marcas) y abre el **diálogo de impresión de Android** (`PdfPrintAdapter.kt`). Ahí se elige la impresora (Wi-Fi, Mopria, el servicio del fabricante o «Guardar como PDF»), las copias y la doble cara. El archivo temporal se borra cuando el sistema lo ha leído |
| **Reconocer texto (OCR)** | Modelos junto al ejecutable | Los modelos van en el APK (`assets/models`, unos 12 MB, sin comprimir). En el primer arranque se copian a la carpeta privada de la app y `PDFCRAFT_MODELS` apunta a ellos. Se usan los ONNX actuales de upstream, comprobados con su SHA-256, o la pareja `.rten` anterior de PdfCraft for Android si S3 no responde. Licencia CC-BY-SA-4.0 (`NOTICE.md`) |
| **Enlaces** (en documentos, menú Ayuda) | Navegador del sistema | Se sacan de la salida de egui y se abren con un `Intent` `ACTION_VIEW`. Solo se abren `http(s)`, `mailto` y `tel`; un enlace a otra cosa, como un archivo del dispositivo, no se sigue |

## OCR en español y otros idiomas (Tesseract)

El motor de OCR propio de PdfCraft (ocrs) solo reconoce inglés: el alfabeto latino sin acentos. En
escritorio, PdfCraft lo es igual (`LANGUAGES = [("en", "English")]`, brecha 7 de
`upstream/docs/gaps.md`). Para más idiomas, PdfCraft tiene un segundo motor: **Tesseract**, que
ejecuta como programa aparte y del que acepta español, francés, alemán, italiano, portugués,
neerlandés, ruso, chino, japonés, coreano y árabe.

- `scripts/build-tesseract-android.sh` compila Tesseract 5.5.4 con Leptonica 1.85.0 y libpng 1.6.49
  para Android, todo estático y con la zlib del sistema, y descarga los datos de idioma de
  `tessdata_fast` en una revisión fija. Por defecto descarga los idiomas latinos (eng spa fra deu
  ita por nld); `DEXPDFCRAFT_TESSDATA_LANGS` cambia la lista.
- Gradle mete el programa como `lib/<abi>/libtesseract.so`, porque Android solo ejecuta programas
  desde la carpeta de librerías nativas de la app, y los datos como `assets/tessdata`.
- Al arrancar, `MainActivity` apunta `PDFCRAFT_TESSERACT` al programa y la librería copia los datos
  y fija `TESSDATA_PREFIX`. PdfCraft lo detecta solo (`--list-langs`) y ofrece esos idiomas en
  Escanear y OCR.
- Sin el programa compilado, el APK se construye igual y el OCR ofrece solo inglés.

## Lo que no hay en Android

- **«Mostrar en la carpeta»** (menú de una pestaña): en escritorio lanza el gestor de archivos;
  Android no ofrece un equivalente general y la orden no hace nada.
- **Compartir desde la app y leer en voz alta:** PdfCraft de escritorio no tiene estas funciones.
  Se pueden añadir en el fork como funciones solo de Android si se quieren.

## Por probar en el dispositivo

1. Archivo ▸ Imprimir: elegir «Android», pulsar Imprimir; se abre el diálogo de Android y se puede
   imprimir o guardar como PDF.
2. Escanear y OCR ▸ Reconocer texto en un PDF escaneado: el texto queda seleccionable.
3. Un enlace web en un documento y Ayuda ▸ (Discord, web): se abren en el navegador.
