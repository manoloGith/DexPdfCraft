package io.github.dexpdfcraft

import android.app.NativeActivity
import android.os.Bundle

/**
 * The whole window is drawn by the Rust library (`libdexpdfcraft.so`, named in the manifest):
 * PdfCraft's egui interface, through android-activity's NativeActivity glue.
 *
 * Phase 3 adds what only Kotlin can do here, called from Rust over JNI: the Storage Access
 * Framework (open, save, save as), "Open with" and share intents, printing and read aloud.
 */
class MainActivity : NativeActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
    }
}
