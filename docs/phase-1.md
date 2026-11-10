# Fase 1: prueba técnica en DeX

La app de la fase 1 abre una ventana con la disposición de PdfCraft en escritorio:

- barra de menús (File, Edit, View, Pages, Help) generada desde el registro de comandos del motor;
- barra rápida (páginas, zoom, escala de interfaz, tema);
- panel «All tools» a la izquierda, con todos los comandos del registro agrupados;
- la página en el centro, renderizada por `pdfcraft-render`, que es el mismo rasterizador del escritorio;
- panel de diagnóstico de entrada a la derecha y barra de estado abajo.

Los comandos del registro solo se anotan en el registro de eventos; los ejecutará la interfaz real en
la fase 2. La navegación y el zoom sí funcionan.

## Qué comprueba

| Prueba | Cómo | Resultado esperado |
|---|---|---|
| Arranque y renderizador | Abrir la app en DeX | La barra de estado muestra `wgpu Vulkan: <GPU>` o `wgpu Gl`; si wgpu falla, `OpenGL ES (glow)` |
| Motor en ARM64 | — | La página del PDF de muestra se dibuja; la barra de estado da el tiempo de render |
| Redimensionar | Arrastrar el borde de la ventana, maximizar, pasar de pantalla del móvil a monitor | La interfaz se adapta sin reiniciar la app; el registro anota `window W×H points` |
| Densidad | Cambiar la escala de DeX o mover la ventana entre pantallas | No se reinicia (`configChanges` y los `meta-data` de Samsung); cambia `px/pt` |
| Hover | Mover el ratón sobre botones | Se resaltan; el panel dice `Hovering: yes` |
| Clic derecho | Sobre la página o el botón de prueba | Menú contextual |
| Clic central, botones atrás/adelante | Sobre el botón de prueba | Aparecen en `Buttons` y en el registro |
| Rueda | Sobre la página | Desplazamiento; con Ctrl, zoom |
| Atajos | Ctrl + + / Ctrl + − / Ctrl + 0, Ctrl + B, Av Pág / Re Pág, Inicio / Fin, flechas | Zoom, ocultar «All tools» y cambiar de página |
| Texto | Escribir en el campo del panel derecho, con acentos y ñ | El texto aparece tal cual |
| Tema | Botones de tema en la barra rápida | Claro, oscuro o el del sistema |
| Segundo plano | Minimizar y volver, bloquear la pantalla | Vuelve a dibujar sin cerrarse (eframe recrea la superficie) |

## Cómo informar

En las pruebas que fallen, anota el modelo del dispositivo, la versión de One UI o Android y la línea
de la barra de estado. Saca también el log:

```bash
adb logcat -s dexpdfcraft RustStdoutStderr AndroidRuntime
```

## Verificado en el contenedor de desarrollo (sin dispositivo)

- `libdexpdfcraft.so` compila para `aarch64-linux-android` (API 29) con cargo-ndk y NDK 29. Exporta
  `android_main`, `GameActivity_onCreate` y `Java_com_google_androidgamesdk_GameActivity_initializeNativeCode`.
- `./gradlew assembleDebug -Pdexpdfcraft.abis=arm64-v8a` genera un APK de 24,8 MB (minSdk 29, targetSdk 36).
- La misma ventana en escritorio (`--features desktop`), bajo Xvfb, arranca wgpu (Vulkan, llvmpipe) y
  renderiza el PDF de muestra con `pdfcraft-render`: ver [images/phase1-desktop.png](images/phase1-desktop.png).
- **No probado aún en un dispositivo DeX**: el contenedor no tiene KVM ni hardware Android. La tabla
  de arriba es la lista que hay que pasar en el dispositivo.

### Cambio respecto al plan: GameActivity en lugar de NativeActivity

eframe solo admite `accesskit` con el backend `android-game-activity`, y `pdfcraft-ui-egui` activa
`accesskit`. GameActivity también trae soporte de IME (GameTextInput), así que la app usa GameActivity
(`androidx.games:games-activity` 4.4.0, la versión que admite android-activity 0.6).

## Tanteo de la fase 2: `pdfcraft-ui-egui` para Android

`cargo ndk -t arm64-v8a check -p pdfcraft-ui-egui` en el workspace de upstream:

- **Todas las crates del motor de las que depende la interfaz compilan para Android** (`engine`, `render`, `sign`, `ocr`, `forms`, `js`…).
- Fallan solo tres dependencias de terceros, todas previstas en el plan:
  - `rfd` (diálogos de archivo y de mensaje): no tiene backend Android. Se sustituye por SAF a través de
    Kotlin (fase 3) y se aísla con `cfg(not(target_os = "android"))` (unos 53 usos de `rfd::` en `ui-egui`).
  - `arboard` (portapapeles de imágenes): no tiene backend Android. `cfg` y el `ClipboardManager` de Android.
  - `android-activity`: necesita que alguien active `game-activity`. Lo hace el crate de la app; aquí
    falla solo porque `ui-egui` se compiló sin ella.
- Como `upstream/` no se modifica, esos `cfg` van en una rama de un fork de PdfCraft o se proponen
  upstream como PR. Es la primera decisión de la fase 2.
