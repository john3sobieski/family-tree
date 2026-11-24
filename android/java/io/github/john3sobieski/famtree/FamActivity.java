package io.github.john3sobieski.famtree;

import android.app.NativeActivity;
import android.content.Context;
import android.content.Intent;
import android.net.Uri;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.KeyEvent;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.TextView;

/**
 * Thin Java layer over NativeActivity: the system file picker and files opened while the app runs.
 * Everything else is Rust. Rust registers {@link #nativeUri} at startup (RegisterNatives).
 */
public class FamActivity extends NativeActivity {
    private static final int PICK_REQUEST = 4711;

    /** Invisible 1x1 input: NativeActivity cannot receive soft-keyboard text itself, so typing goes through this view. */
    private EditText input;
    /** True while Rust sets the text (do not echo it back). */
    private boolean muted;

    /** Implemented in Rust: receives the content URI (as a string) of the file to open. */
    private native void nativeUri(String uri);

    /** Implemented in Rust: true — the app handled "Back" itself (closed the person page or the search). */
    private native boolean nativeBack();

    /** Implemented in Rust: the current text of the search input. */
    private native void nativeText(String text);

    /** Implemented in Rust: the keyboard's "Search" key was pressed. */
    private native void nativeSubmit();

    /**
     * System "Back" (button or gesture). The CI-patched winit reports Back as unhandled, so it reaches here.
     * Rust closes the person page; otherwise the app goes to the background (like a normal launcher app).
     */
    @Override
    public void onBackPressed() {
        boolean handled = false;
        try {
            handled = nativeBack();
        } catch (UnsatisfiedLinkError e) {
            // Rust has not registered the method (yet).
        }
        if (!handled) {
            moveTaskToBack(true);
        }
    }

    /** Called from Rust: shows the soft keyboard; `text` replaces the content of the hidden input. */
    public void showKeyboard(final String text) {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                ensureInput();
                muted = true;
                input.setText(text);
                input.setSelection(input.getText().length());
                muted = false;
                input.requestFocus();
                InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                if (imm != null) {
                    imm.showSoftInput(input, 0);
                }
            }
        });
    }

    /** Called from Rust: hides the soft keyboard. */
    public void hideKeyboard() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                if (input == null) {
                    return;
                }
                InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
                if (imm != null) {
                    imm.hideSoftInputFromWindow(input.getWindowToken(), 0);
                }
                input.clearFocus();
            }
        });
    }

    private void ensureInput() {
        if (input != null) {
            return;
        }
        input = new EditText(this);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        input.setImeOptions(EditorInfo.IME_ACTION_SEARCH | EditorInfo.IME_FLAG_NO_EXTRACT_UI);
        input.setBackground(null);
        input.setAlpha(0f);
        input.addTextChangedListener(new TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence s, int start, int count, int after) {}

            @Override
            public void onTextChanged(CharSequence s, int start, int before, int count) {}

            @Override
            public void afterTextChanged(Editable e) {
                if (muted) {
                    return;
                }
                try {
                    nativeText(e.toString());
                } catch (UnsatisfiedLinkError err) {
                    // Rust has not registered the method (yet).
                }
            }
        });
        input.setOnEditorActionListener(new TextView.OnEditorActionListener() {
            @Override
            public boolean onEditorAction(TextView v, int actionId, KeyEvent event) {
                try {
                    nativeSubmit();
                } catch (UnsatisfiedLinkError err) {
                    // Rust has not registered the method (yet).
                }
                return true;
            }
        });
        addContentView(input, new FrameLayout.LayoutParams(1, 1));
    }

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
