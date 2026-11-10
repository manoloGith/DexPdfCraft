package io.github.dexpdfcraft

import io.github.craftandroid.CraftActivity

/**
 * DexPdfCraft's activity. Everything Android-side (documents through the Storage Access
 * Framework, clipboard, printing, mouse and keyboard, the on-screen keyboard, lifecycle,
 * diagnostics) is craft-android's [CraftActivity]; the library it loads (`libdexpdfcraft.so`,
 * the manifest's `android.app.lib_name`) runs PdfCraft's interface.
 */
class MainActivity : CraftActivity() {
    /** Tesseract, PdfCraft's second OCR engine, when the APK carries it (lib/<abi>/libtesseract.so). */
    override fun nativePrograms(): Map<String, String> = mapOf("PDFCRAFT_TESSERACT" to "libtesseract.so")

    /** PDFs dragged onto the window open in tabs. */
    override fun droppableMimeTypes(): List<String> = listOf("application/pdf")
}
