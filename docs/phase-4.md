# Fase 4: DeX como escritorio

Las fases 1 a 3 dejaron la interfaz de PdfCraft funcionando en Android. Los ajustes para que el
ratón y el teclado de DeX se comporten como en un escritorio (botones, rueda, hover, teclado físico
con su distribución, atajos) se hicieron en las rondas de corrección de la fase 3 (`phase-3.md`).
Esta fase completa lo que un escritorio muestra y Android no hacía.

| Qué | Cómo |
|---|---|
| **Cursor del ratón** | PdfCraft pide un cursor en cada momento: mano sobre enlaces, I sobre texto, flechas dobles en los bordes de paneles y divisiones, cruz al medir, mano agarrando al arrastrar. winit lo ignora en Android, así que `Shell` lo lee tras cada frame y, si cambia, `MainActivity` pone el `PointerIcon` del sistema equivalente en la superficie. La tabla está en `pointer::pointer_icon_type`, con test |
| **Título** | El título de ventana de PdfCraft («documento • — PdfCraft», con • si hay cambios sin guardar) pasa a ser el título de la actividad y la etiqueta de la tarea, que es lo que muestra la barra de tareas de DeX |
| **Arrastrar y soltar** | Los PDF arrastrados a la ventana, desde «Mis archivos» en DeX o desde otra app en pantalla dividida, se abren en pestañas. El permiso de lectura viene con el arrastre (`requestDragAndDropPermissions`). Normalmente un documento soltado no se puede escribir, así que guardar sobre él avisa y sugiere «Guardar como» |
| **Detección de DeX** | `dex=true/false` en el diagnóstico, leído del campo `semDesktopModeEnabled` de One UI. Fuera de Samsung es `false` |

Ya estaba bien y no ha hecho falta tocarlo:

- **Escala.** En DeX, la densidad de 160 dpi da 1 punto por píxel, la misma escala que en un monitor
  de escritorio. PdfCraft tiene además su propio tamaño de texto en Preferencias.
- **Decoraciones de ventana.** La app no dibuja botones de cerrar ni de maximizar propios; la barra
  de la ventana es la de DeX.
- **Ventanas secundarias.** PdfCraft no abre otras ventanas (viewports) que Android no podría crear.

## Por probar en el dispositivo

1. Pasar el ratón por enlaces, texto, divisores de paneles y por la herramienta de medir: el
   cursor cambia.
2. Editar un documento: la barra de tareas de DeX muestra su nombre con • hasta guardar.
3. Arrastrar un PDF desde «Mis archivos» a la ventana: se abre en una pestaña.
