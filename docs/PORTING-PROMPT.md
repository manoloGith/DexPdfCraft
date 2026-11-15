# Prompt para portar una app de storytold a Android y DeX

Copia este documento entero como primer mensaje de una sesión nueva y sustituye `<App>`, `<app>`
y `<owner>`. Recoge todo lo aprendido al portar **PdfCraft → DexPdfCraft**
(https://github.com/manoloGith/DexPdfCraft): la arquitectura que funcionó, el código reutilizable y
cada fallo que encontramos en un dispositivo DeX real, con su causa y su arreglo. El objetivo es no
volver a descubrirlos.

---

## 1. El encargo

Quiero portar **<App>** (https://github.com/storytold/<app>), una app de escritorio de storytold
escrita en Rust con egui/eframe, a **Android**. El objetivo principal es **Samsung DeX** y el modo
escritorio de Android (ratón, teclado y ventanas). También debe funcionar en tabletas y móviles con
el dedo y el teclado en pantalla.

- **Misma interfaz y mismas funciones** que en escritorio, sin rehacer la interfaz: se compila para
  Android la propia interfaz egui de la app y se le añade una capa de plataforma Android.
- Sigue la arquitectura y las lecciones de este documento. **Reutiliza el código de DexPdfCraft**
  (sección 5) en lugar de reescribirlo.
- Trabaja por fases (sección 3), con un commit por paso. Cada fase deja un APK que se puede probar
  y su documento en `docs/phase-N.md`.
- No hay emulador ni dispositivo en el entorno: **di siempre qué se ha probado y qué no**. La app
  lleva desde el principio el informe de diagnóstico integrado (sección 4.10), para que yo pueda
  copiarlo desde el dispositivo sin adb.
- Escríbeme en español. Código, comentarios y commits, en inglés.

## 2. Arquitectura (decidida; no la reabras sin motivo)

```
<App>-Dex/
├── upstream/              submódulo: fork <owner>/<app>, rama dexcraft/android (upstream + pocos commits)
├── craft-android/         submódulo: manoloGith/craft-android (capa Android compartida + winit con W-DEX1..3)
├── apps/<app>-dex/        crate cdylib (la app Android) + bin de escritorio para iterar
│   ├── src/               lib.rs (Shell de craft-android), android/mod.rs (android_main + backends rfd/arboard)
│   └── android/           proyecto Gradle: MainActivity : CraftActivity (+ :craft-activity)
├── scripts/               fetch-build-inputs.sh, sync-upstream.sh
├── fastlane/metadata/     F-Droid / Obtainium
└── .github/workflows/     android.yml (CI), release.yml (firma + release)
```

1. **Fork + submódulo, no copia.** Haz fork de `storytold/<app>` y crea la rama
   `dexcraft/android`. El submódulo `upstream/` apunta a esa rama. Los cambios al upstream deben
   ser **pocos, genéricos y aislados**, para poder rebasarlos y, si se quiere, proponerlos upstream.
   Para actualizar, rebasa sobre el `main` de storytold (`scripts/sync-upstream.sh`).
2. **Workspace Cargo propio** con `exclude = ["upstream", "craft-android"]`.
   - Las crates de upstream se usan como dependencias por ruta y heredan su `workspace = true` del
     workspace de upstream.
   - **`[patch.crates-io]` no se hereda**: hay que **copiar los de upstream** al `Cargo.toml` raíz,
     con las rutas reescritas a `upstream/vendor/...`, y `sync-upstream.sh` comprueba que coinciden.
   - Siembra `Cargo.lock` con el de upstream, para usar las mismas versiones que ellos prueban.
3. **GameActivity, no NativeActivity.** eframe solo admite `accesskit`, que la interfaz de storytold
   activa, con `android-game-activity`. Además, GameActivity trae IME. La dependencia Java es
   `androidx.games:games-activity:4.4.0`, que es la que admite android-activity 0.6. Tema AppCompat.
4. **Kotlin mínimo y JNI mínimo.** Hay un solo método Kotlin que llama Rust,
   `bridge(op: String, json: String, data: ByteArray?): ByteArray?`, y unas pocas funciones
   nativas: `nativeRegister`, `nativeEvent(op, json)`, `nativePointer`, `nativeKey`, `nativeIme`,
   `nativeInsets`. Todo viaja como JSON y bytes (`protocol.rs`, con tests en host).
   - La JVM es `jni::JavaVM::singleton()`. Usa la **misma versión de jni-rs que android-activity**
     (0.22.x); si no, el singleton no existe.
   - El `unsafe` solo aparece en `#[unsafe(no_mangle)]` de las funciones exportadas, en
     `android_main` y en `std::env::set_var`. El resto, `deny(unsafe_code)`.
5. **Shims en lugar de cambiar código de upstream:**
   - `rfd` (diálogos) y `arboard` (portapapeles) no tienen backend Android. Se sustituyen por crates
     shim con el mismo nombre de lib (`package = "rfd-android-shim"`, `[lib] name = "rfd"`). En los
     demás sistemas reexportan la crate real (renombrada `rfd_real`); en Android los responde un
     backend que instala la app.
   - **Cargo prohíbe** que una dependencia tenga fuentes distintas según el target: el shim debe
     ser la dependencia en *todos* los targets.
6. **El entorno que espera la app, sin tocarla.**
   - En `android_main`, antes de crear hilos, apunta `HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`,
     `XDG_CACHE_HOME`, `XDG_STATE_HOME` y `TMPDIR` al almacenamiento privado.
   - `LANG` se toma de `AndroidApp::config()` (por ejemplo `es_ES.UTF-8`): el idioma «Auto» de las
     apps de storytold lo lee.
7. **Arranque como el binario de escritorio:** lee `apps/<app>/src/main.rs` (su `app_creator`) y
   reprodúcelo, quitando lo que en Android no aplica: instancia única, canal de control, Apple
   events, comprobación de actualizaciones, impresoras del sistema, decoraciones de ventana propias.
8. **`Shell`**: un `eframe::App` que envuelve la app y delega todo (`logic`, `ui`, `save`,
   `on_exit`, `auto_save_interval`, `clear_color`, `persist_egui_memory`, `raw_input_hook`). En el
   `raw_input_hook` inyecta los eventos de ratón, teclado, IME e insets de Android.

## 3. Fases (en este orden)

| Fase | Contenido |
|---|---|
| 0 | Fork, submódulo, workspace con los `[patch]` copiados, `vendor/winit` con W-DEX1..3, Gradle + cargo-ndk, CI, NOTICE y licencias, **sin marcas de ArtCraft** |
| 1 | Prueba técnica: eframe en GameActivity, wgpu con glow de reserva, el motor de la app en arm64, ventana de diagnóstico |
| 2 | La interfaz real: shims `rfd`/`arboard` en el fork, arranque como en escritorio, entorno HOME/XDG/LANG, traducciones (craft-i18n), fuentes de `/system/fonts` |
| 3 | Kotlin: SAF con escritura de vuelta, portapapeles, «Abrir con»/«Compartir», **ratón y teclado por la actividad**, ciclo de vida robusto, **informe de diagnóstico** |
| 4 | Escritorio: cursores (`PointerIcon`), título de la tarea, arrastrar y soltar, detección de DeX |
| 5 | Funciones que piden al sistema: imprimir, modelos o assets, URLs con `Intent`, programas externos |
| 6 | Móvil: teclado en pantalla (GameTextInput → IME) e insets (edge-to-edge) |
| 7 | Tamaño (perfil mixto), clave de release, workflow de release, metadatos de F-Droid |

**Mete desde la fase 1 lo que en DexPdfCraft costó varias rondas de prueba en el dispositivo:**
W-DEX1..3, `pointer.rs`, `keyboard.rs`, la terminación del proceso en `onDestroy`, `RestartActivity`
y el diagnóstico. No esperes a que el usuario descubra los fallos.

## 4. Fallos que ya conocemos: síntoma → causa → arreglo

### 4.1 La app se cierra sola al abrirla (a veces, sobre todo la segunda vez)
- **Causa:** Android avisa de un cambio de configuración (DeX amplía la ventana al abrirla) **antes
  de crear la ventana nativa**. winit lo envía como `WindowEvent::ScaleFactorChanged`, y eframe, que
  aún no tiene ventana (`running == None`), devuelve `EventResult::Exit`.
- **Arreglo (W-DEX3, en `vendor/winit`):** en `single_iteration` se envuelve el callback para
  descartar los `WindowEvent` hasta el primer `MainEvent::InitWindow` (estático `WINDOW_CREATED`).

### 4.2 Al cerrar, la app se cuelga y luego no vuelve a abrir
- **Causa:** `GameActivity.onDestroy` espera en el hilo principal a que termine `android_main`
  (`android_app_free` → `pthread_cond_wait`). winit **ignoraba `MainEvent::Destroy`**, así que el
  bucle no acababa nunca.
- **Arreglo (W-DEX2):** `MainEvent::Destroy` → `self.window_target.p.exit()`.

### 4.3 Reabrir dentro del mismo proceso falla
- **Causa:** winit solo deja crear un `EventLoop` por proceso (`EVENT_LOOP_CREATED` →
  `RecreationAttempt`). El thread-local de eframe no sirve porque `android_main` corre en otro hilo.
- **Arreglo:**
  - `onDestroy` (después de `super`) espera a que acaben las escrituras pendientes y hace
    `Process.killProcess(myPid())`.
  - Un `@Volatile processUsed` en el companion detecta un proceso reutilizado. En ese caso se lanza
    `RestartActivity` (con `android:process=":restart"`, al estilo ProcessPhoenix), que mata el
    proceso antiguo y vuelve a abrir la actividad en uno nuevo.
  - En Rust, un `AtomicBool` hace que un segundo `android_main` retorne sin tocar winit.

### 4.4 El ratón no funciona: rueda, hover, clic derecho, o incluso el izquierdo
- **Causa:** el backend Android de winit convierte **cualquier** evento de puntero en `Touch`,
  ignora `ACTION_SCROLL` y el hover, y no conoce los botones. Encima, GameActivity solo copia los
  ejes X/Y salvo que se pidan otros (`enable_motion_axis`).
- **Arreglo que funcionó (no lo arregles dentro de winit):** en `MainActivity`, después de
  `super.onCreate`, pon tus propios listeners en `mSurfaceView` (es `protected`):
  `setOnTouchListener` y `setOnGenericMotionListener`.
  - Si `isMouse(e)` (`SOURCE_MOUSE`, `SOURCE_TOUCHPAD` o `TOOL_TYPE_MOUSE`), lo atiendes tú y llamas
    a `nativePointer(gone, x, y, buttons, hscroll, vscroll, metaState)`.
  - Si no, llamas a `processMotionEvent(e)`: el táctil sigue funcionando como antes.
  - Botones: `buttonState & 0x1f`; si viene a 0, `DOWN` es el primario y
    `BUTTON_PRESS`/`RELEASE` usan `actionButton`.
  - `pointer.rs` lo convierte en `PointerMoved`, `PointerButton` (los cambios respecto al estado
    anterior), `MouseWheel{Line}` y `PointerGone`.
  - Registro real de DeX (S24): `src=0x2002 tool=3`, `HOVER_EXIT` → `DOWN` → `BUTTON_PRESS`… con
    `buttonState` correcto.

### 4.5 Del teclado físico solo funcionan F1–F12, las flechas y Re Pág/Av Pág
- **Causa:** winit Android traduce por tabla las teclas que conoce, pero no obtiene el carácter de
  letras, números y símbolos.
- **Arreglo:** `dispatchKeyEvent` en `MainActivity` para teclados físicos
  (`!device.isVirtual && SOURCE_KEYBOARD`, sin Back, volumen, Home ni Menu).
  - El texto sale de `getUnicodeChar(metaState)`, que ya aplica Mayús y AltGr. Las teclas muertas
    se resuelven con `COMBINING_ACCENT` + `KeyCharacterMap.getDeadChar`.
  - La tecla lógica sale de `getUnicodeChar(0)`, porque la distribución decide.
  - Se llama a `nativeKey(keycode, base, text, pressed, repeat, meta)`, y `keyboard.rs` produce
    `Key{logical, physical}` y `Text` (sin texto si hay Ctrl o es un carácter de control).
  - **egui 0.37 (fork de storytold) no tiene `RawInput.modifiers`**: los modificadores se comunican
    con `Event::ModifiersChanged` cuando cambian.
  - Atajo propio: Ctrl+Mayús+D abre el diagnóstico.

### 4.6 En el móvil no se puede escribir con el teclado en pantalla
- **Causa:** GameActivity da al IME un buffer propio (GameTextInput) y avisa con el estado
  completo (`TextEvent`); winit lo descarta.
- **Arreglo:** sobrescribe `stateChanged(State, Boolean)`, sin llamar a `super`, y pasa el estado
  con `nativeIme(text, composingStart, composingEnd, reset)`.
  - `ime.rs` compara con el estado anterior: lo confirmado se escribe como `Text` (con `Backspace`
    para lo borrado y `\n` como Intro), y lo que se está componiendo va en
    `ImeEvent::Preedit`.
  - Tras cada confirmación, `setTextInputState(State("",0,0,-1,-1))` vacía el buffer; un flag evita
    que eso vuelva a entrar en `stateChanged`.
  - **Las posiciones de Java son UTF-16**: hay que convertirlas a índices de carácter.

### 4.7 La interfaz queda debajo de la barra de estado, la de navegación o el notch
- **Causa:** con targetSdk 35+ Android dibuja la app de borde a borde.
- **Arreglo:** sobrescribe `onApplyWindowInsets` y pasa
  `systemBars | displayCutout | ime` a `nativeInsets`. En `raw_input_hook`, asígnalos a
  `raw_input.safe_area_insets` (`egui::SafeAreaInsets(MarginF32)`, en puntos).

### 4.8 Al cerrar no pregunta si guardar los cambios
- **Causa:** no llega ningún `close_requested`, y la X de DeX la gestiona el sistema: **no se
  puede interceptar**.
- **Arreglo:**
  - Un `OnBackPressedCallback` llama a `nativeEvent("close")`; `Shell` hace
    `ctx.send_viewport_cmd(ViewportCommand::Close)` y la app puede cancelarlo y preguntar.
  - Cuando eframe sale del bucle, `android_main` llama a `bridge("finish")`, que hace
    `finishAndRemoveTask`.
  - Para la X: `Shell` llama a la función de **autoguardado o recuperación** de la app cada vez que
    la ventana pierde el foco (en PdfCraft, `autosave_now()`), y el siguiente arranque ofrece
    recuperar el trabajo.

### 4.9 Cursor, título y enlaces no hacen nada
- winit Android ignora `set_cursor` y `set_title`. Tras `app.ui()`, `Shell` lee
  `ctx.output(|o| o.cursor_icon)` y el título de la app; si cambian, `bridge("cursor")`
  (`PointerIcon.getSystemIcon` sobre `mSurfaceView`) y `bridge("title")`
  (`setTaskDescription`). La tabla de egui → `PointerIcon.TYPE_*` está en `pointer.rs`.
- Las URLs: retira `OutputCommand::OpenUrl` de `o.commands` en Android y ábrelas con un `Intent
  ACTION_VIEW`, solo `http(s)`, `mailto` y `tel`.

### 4.10 Sin adb no hay forma de saber qué pasa → diagnóstico integrado desde el principio
- `Diag.kt` mantiene `files/diag.log`: ciclo de vida, configuración (pantalla, densidad, uiMode,
  display, DeX), los primeros 60 eventos de ratón en bruto y las primeras 20 teclas (**solo
  códigos, nunca el texto**). `rust: diag()` añade hitos al mismo archivo.
- El hook de panic escribe `crash.txt`, y un `UncaughtExceptionHandler` hace lo mismo para Java.
- Si la sesión anterior no terminó con `session end`, o hay un crash, el siguiente arranque muestra
  un diálogo con el informe y un botón **Copiar informe**.
- También se abre con Ctrl+Mayús+D, con un acceso directo dinámico del icono («Diagnóstico») y, en
  debug, con un icono aparte (`activity-alias` en `src/debug/AndroidManifest.xml`).
- **Con esto se resolvieron 4.1 y 4.4**: el usuario pegó el informe y se vio la causa.

### 4.11 Archivos: la app trabaja con rutas y Android con documentos SAF
- **Arreglo (`DocumentStore.kt`):** cada documento se copia a
  `files/documents/<hash del URI>/<nombre>`. El mismo URI siempre da la misma ruta, así que no se
  duplican pestañas ni recientes.
- Un `FileObserver(CLOSE_WRITE | MOVED_TO)` por carpeta escribe de vuelta cada guardado.
  - Las apps de storytold guardan de forma atómica: escriben `.<nombre>…tmp` y lo renombran. **Hay
    que ignorar los nombres que empiezan por `.`**.
  - Para no reescribir el documento al abrirlo, se guarda la marca de lo último sincronizado
    (`lastModified`, `length`), sin flags.
- `takePersistableUriPermission` y el mapa ruta → URI en `SharedPreferences` hacen que guardar
  funcione tras reiniciar.
- «Guardar como» (`CREATE_DOCUMENT`) y las carpetas (`OPEN_DOCUMENT_TREE`) usan el mismo mecanismo.
- En `onDestroy`, `flush()`: 300 ms de margen y luego esperar al executor, con un máximo de 4 s
  (más bloquearía el hilo principal).

### 4.12 Programas externos y assets grandes
- Android solo ejecuta binarios desde `nativeLibraryDir`. Inclúyelos como `lib/<abi>/libX.so` y
  pon `jniLibs.useLegacyPackaging = true` para que se extraigan al instalar.
- Pasa su ruta con `android.system.Os.setenv(...)` **antes de `super.onCreate`**.
- Modelos y datos: en `assets/`, sin comprimir (`noCompress`). En el primer arranque se copian al
  almacenamiento privado (`AndroidApp::asset_manager().open_dir/open`, de forma atómica, solo si
  cambia el tamaño) y se apunta la variable de entorno que lea la app.

### 4.13 Licencia y marcas
- Los logotipos de ArtCraft (`docs/brand/`) **no se pueden usar en versiones modificadas**. En el
  fork, las funciones que los dibujan muestran texto o un espacio vacío.
- Tampoco se usa el icono de la app original; crea uno propio.
- «Acerca de»: añade al fork un enganche genérico `product` (nombre, versión, texto por idioma,
  enlaces) que la app rellena, con el agradecimiento a la app original y a sus autores. Ver
  `upstream/crates/ui-egui/src/product.rs` del fork de PdfCraft.

## 5. Código que se reutiliza: craft-android

Todo lo de la sección 4 que no depende de la app está en
[manoloGith/craft-android](https://github.com/manoloGith/craft-android), que se añade como
submódulo `craft-android/` (guía: `craft-android/docs/integrate.md`). **No lo copies: úsalo**, y si
algo falla en una app, arréglalo allí para todas.

| Pieza de craft-android | Para qué | Qué pone la app |
|---|---|---|
| `winit/` (+ `PATCHES.md`, tests) | W-DEX1 (ratón), W-DEX2 (Destroy), W-DEX3 (eventos antes de la ventana) | `winit = { path = "craft-android/winit" }` en `[patch.crates-io]`; comprobar que upstream fija la rev `a517b38…` |
| `craft_android::Shell<A>` | Entrada de la actividad (ratón, teclado, IME, insets), Back = cerrar, autoguardado al perder el foco, cursor, título, URLs | `.on_focus_lost(\|a\| a.<autoguardado>())`, `.title(\|a\| …)` |
| `craft_android::android::{start, native_options, finish, diag}` | Entorno (`HOME`, XDG, `LANG`), pánicos y diagnóstico, guardia de `STARTED`, ejes de la rueda, assets del APK | `Config { log_tag, asset_dirs }` |
| `android::{pickers, clipboard, print, events}` | SAF, portapapeles, diálogo de impresión, «Abrir con» | Los adaptadores a `rfd::Backend`/`arboard::Backend` (copiar de DexPdfCraft `apps/dexpdfcraft/src/android/mod.rs`) y el evento de apertura de la app |
| `crates/{rfd,arboard}-android-shim` | Shims con backend instalable | Que la crate de interfaz del fork dependa de ellos en todos los objetivos |
| `android/craft-activity` (`CraftActivity`, `DocumentStore`, `ClipboardBridge`, `Diag`, `RestartActivity`, `PdfPrintAdapter`) | Todo el lado Kotlin; `RestartActivity` y el `FileProvider` ya van en su manifiesto | `class MainActivity : CraftActivity()` con `nativePrograms()` y `droppableMimeTypes()` |

De DexPdfCraft se copian además (adaptando nombres): `android/app/build.gradle.kts` (cargo-ndk,
assets, firma), el `AndroidManifest.xml` de la app (`configChanges`, `layout`, meta-data de
Samsung, intent filters con los tipos MIME de la app, alias de diagnóstico en debug),
`scripts/fetch-build-inputs.sh` y `sync-upstream.sh`, `.github/workflows/{android,release}.yml`,
`docs/releasing.md`, `fastlane/metadata/android/` y `docs/fdroid.md`.

## 6. Qué investigar en cada app nueva antes de escribir código

1. `apps/<app>/src/main.rs`: el `app_creator`, los campos que rellena (fuentes del sistema,
   `os_events`, impresión, recuperación…) y las `NativeOptions`.
2. `crates/ui-*/Cargo.toml`: `rfd`, `arboard`, `craft-update` y cualquier dependencia con código
   C (`cc`, cmake) que pueda no compilar para Android. Haz pronto un
   `cargo ndk -t arm64-v8a check -p <ui-crate>` en el workspace de upstream (con
   `CARGO_TARGET_DIR` aparte) para ver la lista real de fallos.
3. `std::process::Command::new`: qué lanza (abrir carpeta, herramientas externas) y qué hacer en
   Android.
4. Viewports secundarios (`show_viewport_immediate` o `deferred`): en Android no hay segunda
   ventana.
5. Assets y entradas de compilación: craft-i18n, craft-fonts, modelos de ML. Cuánto pesan.
6. Marcas: `include_bytes!` de `docs/brand/` e iconos de la app.
7. Rutas de datos y de configuración: dónde las calcula, para que HOME/XDG baste.
8. Formatos que abre, para los intent filters `VIEW`/`SEND`.
9. Cómo autoguarda o recupera, para el autoguardado al perder el foco.
10. Funciones de hardware o del sistema (audio en SoundCraft y FilmCraft, cámara, GPU): cada una es
    trabajo extra en Android.

## 7. Entorno de compilación y git (lo que nos frenó)

- **Red:** el SDK y el NDK se descargan de `dl.google.com`, que hay que añadir al entorno. Maven y
  GitHub (`git clone` y `raw.githubusercontent.com`) funcionaban. Las URLs `github.com/.../archive/`
  y `/raw/` devolvían 403 a través del proxy.
- **Versiones que funcionan:** NDK 29.0.14206865, plataforma 36 (minSdk 29, targetSdk 36),
  cargo-ndk 4.1.2, AGP 9.2.1 con Gradle 9.5.1 (wrapper copiado de pdfcraft-android), JDK 17,
  Rust 1.97.
- **Disco:** el contenedor tiene una cuota; `target/` de un workspace así llega a 20 GB. Borra los
  `target/*/incremental` y los directorios de prueba (`CARGO_TARGET_DIR` aparte) cuando no hagan
  falta. «No space left on device» en LLVM no es un error del código.
- **Submódulo:** nunca hagas commit del gitlink apuntando a un commit del fork que no esté subido,
  porque el clon recursivo falla. Mientras trabajas en el fork: `git add -A -- ':!upstream'`, y sube
  la rama del fork antes de registrar el submódulo. Comprueba siempre con un clon nuevo
  `--recursive`.
- **Tags:** el proxy de git de la sesión no deja subir tags. El workflow de release acepta
  `workflow_dispatch`, que se lanza con la API de GitHub, y crea él mismo el tag `v<versión>` con
  `softprops/action-gh-release` (`tag_name` + `target_commitish`).
- **Versión nueva:** después de subirla en `Cargo.toml`, ejecuta `cargo update -w`, porque el build
  usa `--locked`. Si no, el release falla en un minuto.
- **Secretos pegados:** el workflow quita espacios y saltos de línea. Si el alias no existe en el
  almacén, usa su única clave; en PKCS12 la contraseña de la clave es la del almacén.
- **Código de terceros:** descargar y compilar fuentes externas (por ejemplo Tesseract) lo bloquea
  la protección de la sesión: hay que pedirme aprobación.
- **Tamaño:** `[profile.release.package."*"] opt-level = "s"`, salvo las crates que dibujan,
  decodifican o calculan, que van a `opt-level = 3`. En PdfCraft fue −16 % de librería comprimida.

## 8. Prueba en el dispositivo (DeX y móvil)

Pide al usuario, en este orden, y que pegue el informe (Ctrl+Mayús+D) si algo falla:

1. Abrir, cerrar con la X y volver a abrir **tres veces seguidas**.
2. Ratón: clic izquierdo, derecho, central y rueda (y Ctrl+rueda) sobre el documento y los menús.
   Cursor sobre enlaces, texto y bordes.
3. Teclado físico: letras, números, símbolos con AltGr, acentos (´ + a), ñ, atajos con Ctrl.
4. Archivo ▸ Abrir con el selector de Android, editar, Ctrl+S: el cambio aparece en el original.
   «Guardar como», «Abrir con» desde «Mis archivos» y arrastrar y soltar.
5. Atrás con cambios sin guardar: debe preguntar.
6. En el móvil: el teclado en pantalla con predicción, que la interfaz no quede bajo las barras, y
   el giro de pantalla.
