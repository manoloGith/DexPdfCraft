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

## Resultados de compilación (en el contenedor de desarrollo)

Ver la sección «Verificado» del commit de la fase 1.
