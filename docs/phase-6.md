# Fase 6: móvil y tablet sin DeX

En el móvil, con el dedo, la interfaz ya funcionaba: tocar, seleccionar, el menú contextual con una
pulsación larga, desplazarse. Faltaba lo que el táctil no prueba.

| Qué | Problema | Arreglo |
|---|---|---|
| **Escribir con el teclado en pantalla** | GameActivity da al teclado un buffer propio (GameTextInput) y avisa de cada cambio con el estado completo; winit lo descartaba, así que lo escrito no llegaba a la interfaz. Solo Intro y Retroceso, que llegan como teclas, funcionaban | `MainActivity.stateChanged` pasa cada estado a `src/ime.rs`, que lo compara con el anterior. El texto que pasa a ser definitivo se escribe, con Retroceso para lo borrado y saltos de línea como Intro. La palabra que el teclado predictivo aún está componiendo se muestra como pre-edición de egui. Tras cada confirmación se vacía el buffer, para que el teclado no reescriba palabras ya escritas. Las posiciones de Java son UTF-16 y se convierten a caracteres. Tests en `ime.rs` |
| **Barras del sistema, notch y teclado** | Con targetSdk 36, Android 15+ dibuja la app de borde a borde: la barra de estado, la de navegación o el notch podían tapar los menús o la barra inferior, y el teclado en pantalla el campo en el que se escribe | `onApplyWindowInsets` pasa a `src/insets.rs` lo que cubre el sistema (barras, recorte de pantalla y teclado), y `Shell` se lo da a egui como `safe_area_insets`: los paneles quedan fuera de esa zona. En una ventana de DeX suele ser cero |

Un teclado físico conectado al móvil o la tablet (USB o Bluetooth) usa la misma ruta que en DeX
(`keyboard.rs`), y un ratón también (`pointer.rs`).

## Por probar

1. En el móvil, tocar «Buscar herramientas y comandos» o un campo de formulario: sale el teclado.
   Escribir con predicción, acentos, emoji y borrar: el texto aparece en el campo.
2. La interfaz no queda debajo de la barra de estado ni de la de navegación, y al abrir el teclado
   el campo sigue visible.
3. Girar el móvil: la interfaz se adapta.

## Fuera de esta fase

- **La interfaz es la de escritorio** a escala de móvil: cabe, pero en una pantalla de unos 400 dp
  de ancho los paneles van justos. PdfCraft tiene en Preferencias su propio tamaño de interfaz. Un
  diseño específico para móvil sería otro proyecto: es lo que hace PdfCraft for Android.
