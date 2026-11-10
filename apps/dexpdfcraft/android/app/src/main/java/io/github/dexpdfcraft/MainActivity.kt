package io.github.dexpdfcraft

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.widget.Toast
import com.google.androidgamesdk.GameActivity
import org.json.JSONArray
import org.json.JSONObject
import java.util.concurrent.Executors

/**
 * The whole window is drawn by the Rust library (`libdexpdfcraft.so`, named in the manifest):
 * PdfCraft's egui interface, through android-activity's GameActivity glue (GameActivity rather
 * than NativeActivity for its IME support, and because eframe's accesskit needs it).
 *
 * This class is the library's way to what only Android offers. Rust calls [bridge] over JNI; the
 * answers that come later (a picked file, a document opened with the app) go back through
 * [nativeEvent]. Everything crosses as JSON text and byte arrays, so the JNI surface stays at three
 * methods.
 */
class MainActivity : GameActivity() {
    private lateinit var documents: DocumentStore
    private lateinit var clipboard: ClipboardBridge
    private val io = Executors.newSingleThreadExecutor()

    /** Pickers on screen: request code → (Rust's request id, kind). */
    private val pending = HashMap<Int, Pair<Long, Int>>()
    private var nextRequest = 0

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState) // loads libdexpdfcraft.so
        current = this
        documents = DocumentStore(applicationContext).also { it.start() }
        clipboard = ClipboardBridge(applicationContext)
        nativeRegister(this)
        if (savedInstanceState == null) handleIntent(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    override fun onDestroy() {
        if (current === this) current = null
        super.onDestroy()
    }

    /** "Open with" and "Share" from other apps: open the PDFs in a new tab. */
    private fun handleIntent(intent: Intent?) {
        val uris = when (intent?.action) {
            Intent.ACTION_VIEW, Intent.ACTION_EDIT -> listOfNotNull(intent.data)
            Intent.ACTION_SEND -> listOfNotNull(intent.streamExtra())
            Intent.ACTION_SEND_MULTIPLE -> intent.streamExtras()
            else -> emptyList()
        }
        if (uris.isEmpty()) return
        val writable = intent!!.flags and Intent.FLAG_GRANT_WRITE_URI_PERMISSION != 0
        io.execute {
            val paths = uris.mapNotNull { documents.open(it, writable) }
            if (paths.size < uris.size) toast(getString(R.string.open_failed))
            if (paths.isNotEmpty()) nativeEvent("open", JSONObject().put("paths", JSONArray(paths)).toString())
        }
    }

    /**
     * Called by Rust (any thread) with an operation, its JSON arguments and optional bytes; returns
     * bytes or null. Never throws: a failure is logged and answered with null.
     */
    @Suppress("unused") // called over JNI
    fun bridge(op: String, json: String, data: ByteArray?): ByteArray? = try {
        val args = if (json.isEmpty()) JSONObject() else JSONObject(json)
        when (op) {
            "pick" -> { pick(args); null }
            "clip_get" -> clipboard.get().toString().toByteArray()
            "clip_image" -> clipboard.image()
            "clip_text" -> { clipboard.setText(args.getString("text")); null }
            "clip_html" -> { clipboard.setHtml(args.getString("html"), args.optString("text")); null }
            "clip_set_image" -> { clipboard.setImage(args.getInt("width"), args.getInt("height"), data ?: ByteArray(0)); null }
            "clip_files" -> { clipboard.setFiles(args.getJSONArray("paths")); null }
            "toast" -> { toast(args.getString("text")); null }
            else -> { Log.w(TAG, "bridge: unknown operation $op"); null }
        }
    } catch (e: Exception) {
        Log.w(TAG, "bridge: $op failed", e)
        null
    }

    /** Show the system picker for a request from the rfd shim (see `Kind` in its Rust code). */
    private fun pick(args: JSONObject) {
        val id = args.getLong("id")
        val kind = args.getInt("kind")
        val intent = when (kind) {
            KIND_FOLDER, KIND_FOLDERS -> Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
            KIND_SAVE -> Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                val name = args.optString("file_name").ifEmpty { "document.pdf" }
                type = mimeTypes(args).firstOrNull() ?: DocumentStore.mimeFor(name)
                putExtra(Intent.EXTRA_TITLE, name)
            }
            else -> Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                val mimes = mimeTypes(args)
                type = mimes.singleOrNull() ?: "*/*"
                if (mimes.size > 1) putExtra(Intent.EXTRA_MIME_TYPES, mimes.toTypedArray())
                putExtra(Intent.EXTRA_ALLOW_MULTIPLE, kind == KIND_OPEN_FILES)
            }
        }
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        runOnUiThread {
            val code = REQUEST_BASE + (nextRequest++ and 0xfff)
            pending[code] = Pair(id, kind)
            try {
                @Suppress("DEPRECATION")
                startActivityForResult(intent, code)
            } catch (e: Exception) {
                Log.w(TAG, "no picker for $intent", e)
                pending.remove(code)
                answer(id, emptyList())
            }
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        @Suppress("DEPRECATION")
        super.onActivityResult(requestCode, resultCode, data)
        val (id, kind) = pending.remove(requestCode) ?: return
        val uris = if (resultCode == RESULT_OK && data != null) {
            val clip = data.clipData
            if (clip != null) (0 until clip.itemCount).map { clip.getItemAt(it).uri } else listOfNotNull(data.data)
        } else {
            emptyList()
        }
        io.execute {
            val paths = when (kind) {
                KIND_FOLDER, KIND_FOLDERS -> uris.map { documents.folder(it) }
                KIND_SAVE -> uris.map { documents.created(it) }
                else -> uris.mapNotNull { documents.open(it, writable = true) }
            }
            if (paths.size < uris.size) toast(getString(R.string.open_failed))
            answer(id, paths)
        }
    }

    private fun answer(id: Long, paths: List<String>) =
        nativeEvent("picked", JSONObject().put("id", id).put("paths", JSONArray(paths)).toString())

    /** The MIME types of the dialog's filters (`[[name, [ext, …]], …]`). */
    private fun mimeTypes(args: JSONObject): List<String> {
        val filters = args.optJSONArray("filters") ?: return emptyList()
        val out = LinkedHashSet<String>()
        for (i in 0 until filters.length()) {
            val exts = filters.optJSONArray(i)?.optJSONArray(1) ?: continue
            for (j in 0 until exts.length()) {
                val ext = exts.getString(j)
                if (ext == "*") return emptyList()
                out.add(DocumentStore.mimeFor("x.$ext"))
            }
        }
        return out.filter { it != "application/octet-stream" }
    }

    /** This activity, for the library (stored as a global reference on the Rust side). */
    private external fun nativeRegister(activity: MainActivity)

    /** An event for the library: `picked` `{id, paths}` or `open` `{paths}`. */
    private external fun nativeEvent(op: String, json: String)

    companion object {
        private const val TAG = "dexpdfcraft"
        private const val REQUEST_BASE = 0x4000

        // The rfd shim's `Kind`, in its order.
        const val KIND_OPEN_FILE = 0
        const val KIND_OPEN_FILES = 1
        const val KIND_FOLDER = 2
        const val KIND_FOLDERS = 3
        const val KIND_SAVE = 4

        @Volatile private var current: MainActivity? = null

        fun toast(text: String) {
            val activity = current ?: return
            activity.runOnUiThread { Toast.makeText(activity, text, Toast.LENGTH_LONG).show() }
        }
    }
}

@Suppress("DEPRECATION")
private fun Intent.streamExtra(): Uri? = getParcelableExtra(Intent.EXTRA_STREAM)

@Suppress("DEPRECATION")
private fun Intent.streamExtras(): List<Uri> = getParcelableArrayListExtra<Uri>(Intent.EXTRA_STREAM).orEmpty()
