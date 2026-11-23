package io.github.john3sobieski.famtree;

import android.app.NativeActivity;
import android.content.Intent;
import android.net.Uri;

/**
 * Thin Java layer over NativeActivity: the system file picker and files opened while the app runs.
 * Everything else is Rust. Rust registers {@link #nativeUri} at startup (RegisterNatives).
 */
public class FamActivity extends NativeActivity {
    private static final int PICK_REQUEST = 4711;

    /** Implemented in Rust: receives the content URI (as a string) of the file to open. */
    private native void nativeUri(String uri);

    /** Called from Rust: opens the system file picker. */
    public void pickFile() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                intent.addCategory(Intent.CATEGORY_OPENABLE);
                intent.setType("*/*");
                try {
                    startActivityForResult(intent, PICK_REQUEST);
                } catch (Exception e) {
                    // No file picker on this device: nothing to do.
                }
            }
        });
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode == PICK_REQUEST && resultCode == RESULT_OK && data != null && data.getData() != null) {
            deliver(data.getData());
        }
    }

    /** A file was opened from a file manager while the app was already running. */
    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        if (intent != null && intent.getData() != null) {
            deliver(intent.getData());
        }
    }

    private void deliver(Uri uri) {
        try {
            nativeUri(uri.toString());
        } catch (UnsatisfiedLinkError e) {
            // Rust has not registered the method (yet).
        }
    }
}
