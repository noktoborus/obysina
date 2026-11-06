package ru.styxheim.obusina;

import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;

import com.google.androidgamesdk.GameActivity;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * The activity of Obusina: the game activity; the choice of a document of
 * the system, which Rust asks for by pickPicture or pickPatterns and is
 * answered by picturePicked; and a new document, which Rust asks for by
 * saveDocument and is answered by fileSaved.
 */
public class MainActivity extends GameActivity {
    /** The kinds of picture Obusina reads. */
    private static final String[] KINDS = {"image/png", "image/jpeg", "image/webp"};
    /** The kinds of a file of patterns: any, since xz has no kind of its own everywhere. */
    private static final String[] PATTERNS = {"*/*"};
    /** The kind of a file Obusina saves. */
    private static final String SAVED_KIND = "application/octet-stream";
    /** The size of a chunk read from a document. */
    private static final int CHUNK = 1 << 16;

    /** The choice of a document; its answer goes to opened. */
    private final ActivityResultLauncher<String[]> choice =
            registerForActivityResult(new ActivityResultContracts.OpenDocument(), this::opened);

    /** A new document; its answer goes to created. */
    private final ActivityResultLauncher<String> creation =
            registerForActivityResult(
                    new ActivityResultContracts.CreateDocument(SAVED_KIND), this::created);

    /** The bytes of the document being created, until it is. */
    private byte[] saving;

    /** Opens the choice of a picture; called from Rust, on any thread. */
    public void pickPicture() {
        runOnUiThread(() -> choice.launch(KINDS));
    }

    /** Opens the choice of a file of patterns; called from Rust, on any thread. */
    public void pickPatterns() {
        runOnUiThread(() -> choice.launch(PATTERNS));
    }

    /** Asks where to save bytes as a new document; called from Rust, on any thread. */
    public void saveDocument(String name, byte[] bytes) {
        runOnUiThread(() -> {
            saving = bytes;
            creation.launch(name);
        });
    }

    /** The document created, or null when the person gave it up. */
    private void created(Uri uri) {
        byte[] bytes = saving;
        saving = null;
        if (uri == null || bytes == null) {
            fileSaved(-1);
            return;
        }
        try (OutputStream output = getContentResolver().openOutputStream(uri)) {
            if (output == null) {
                throw new IOException("no stream");
            }
            output.write(bytes);
            fileSaved(1);
        } catch (IOException | SecurityException error) {
            fileSaved(0);
        }
    }

    /** The document the person chose, or null when the choice was given up. */
    private void opened(Uri uri) {
        if (uri == null) {
            picturePicked(null, null);
            return;
        }
        try (InputStream input = getContentResolver().openInputStream(uri)) {
            picturePicked(read(input), name(uri));
        } catch (IOException | SecurityException error) {
            picturePicked(null, null);
        }
    }

    /** All the bytes of a stream. */
    private static byte[] read(InputStream input) throws IOException {
        if (input == null) {
            throw new IOException("no stream");
        }
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        byte[] chunk = new byte[CHUNK];
        int count;
        while ((count = input.read(chunk)) != -1) {
            bytes.write(chunk, 0, count);
        }
        return bytes.toByteArray();
    }

    /** The name the provider gives a document, or null. */
    private String name(Uri uri) {
        String[] columns = {OpenableColumns.DISPLAY_NAME};
        try (Cursor cursor = getContentResolver().query(uri, columns, null, null, null)) {
            if (cursor != null && cursor.moveToFirst()) {
                return cursor.getString(0);
            }
        }
        return null;
    }

    /** The answer of the choice: the bytes and the name, or two nulls. */
    private static native void picturePicked(byte[] bytes, String name);

    /** The answer of a save: 1 written, 0 not written, -1 given up. */
    private static native void fileSaved(int state);
}
