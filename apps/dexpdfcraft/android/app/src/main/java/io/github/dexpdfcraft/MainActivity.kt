package io.github.dexpdfcraft

import android.os.Bundle
import com.google.androidgamesdk.GameActivity

/**
 * The whole window is drawn by the Rust library (`libdexpdfcraft.so`, named in the manifest):
 * PdfCraft's egui interface, through android-activity's GameActivity glue (GameActivity rather
 * than NativeActivity for its IME support, and because eframe's accesskit needs it).
 *
 * Phase 3 adds what only Kotlin can do here, called from Rust over JNI: the Storage Access
 * Framework (open, save, save as), "Open with" and share intents, printing and read aloud.
 */
class MainActivity : GameActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }
}
