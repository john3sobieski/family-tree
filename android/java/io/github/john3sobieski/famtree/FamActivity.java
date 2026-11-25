package io.github.john3sobieski.famtree;

import android.app.NativeActivity;
import android.content.Context;
import android.content.Intent;
import android.content.res.Configuration;
import android.content.res.Resources;
import android.graphics.Insets;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.LocaleList;
import android.text.Editable;
import android.text.InputType;
import android.text.TextWatcher;
import android.view.DisplayCutout;
import android.view.KeyEvent;
import android.view.View;
import android.view.ViewTreeObserver;
import android.view.WindowInsets;
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

    /** Implemented in Rust: system dark mode and the window insets (px) that overlap our content. */
    private native void nativeEnv(boolean dark, int left, int top, int right, int bottom);

    /** Last values sent to Rust (do not repeat the same call). */
    private String lastEnv = "";

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        // Layout changes (rotation, keyboard, bars) change how much of the system UI overlaps us.
        getWindow().getDecorView().getViewTreeObserver().addOnGlobalLayoutListener(new ViewTreeObserver.OnGlobalLayoutListener() {
            @Override
            public void onGlobalLayout() {
                pushEnv(false);
            }
        });
    }

    /** Dark/light switch of the system (the manifest lists uiMode, so the activity is not recreated). */
    @Override
    public void onConfigurationChanged(Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        pushEnv(false);
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            pushEnv(false);
        }
    }

    /** Called from Rust: system UI languages in priority order, e.g. "ru,en". */
    public String languages() {
        LocaleList list = Resources.getSystem().getConfiguration().getLocales();
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < list.size(); i++) {
            if (i > 0) {
                sb.append(',');
            }
            sb.append(list.get(i).getLanguage());
        }
        return sb.toString();
    }

    /** Called from Rust: true if the system is in dark mode. */
    public boolean isNight() {
        int mode = getResources().getConfiguration().uiMode & Configuration.UI_MODE_NIGHT_MASK;
        return mode == Configuration.UI_MODE_NIGHT_YES;
    }

    /** Called from Rust once its callbacks are registered: send the current dark mode and insets. */
    public void requestEnv() {
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                pushEnv(true);
                // The window may not be attached yet, and not every layout pass fires a callback: retry a few times.
                final View decor = getWindow().getDecorView();
                for (int delay : new int[] {150, 600, 1500}) {
                    decor.postDelayed(new Runnable() {
                        @Override
                        public void run() {
                            pushEnv(false);
                        }
                    }, delay);
                }
            }
        });
    }

    /** Height of a system bar from the framework resources (0 if unknown). */
    private int barSize(String name) {
        int id = Resources.getSystem().getIdentifier(name, "dimen", "android");
        return id > 0 ? Resources.getSystem().getDimensionPixelSize(id) : 0;
    }

    /** UI thread. Sends dark mode and the insets that really overlap our content (system bars minus what the window already avoids). */
    private void pushEnv(boolean force) {
        int l = 0, t = 0, r = 0, b = 0;
        View decor = getWindow().getDecorView();
        if (decor.getWidth() > 0) {
            int il = 0, it = 0, ir = 0, ib = 0;
            WindowInsets wi = decor.getRootWindowInsets();
            if (wi != null) {
                if (Build.VERSION.SDK_INT >= 30) {
                    // Ignoring visibility: a window flag may hide the bar's insets while it is still drawn over us.
                    Insets s = wi.getInsetsIgnoringVisibility(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
                    il = s.left;
                    it = s.top;
                    ir = s.right;
                    ib = s.bottom;
                } else {
                    // Stable insets do not depend on visibility and leave out the keyboard.
                    il = wi.getStableInsetLeft();
                    it = wi.getStableInsetTop();
                    ir = wi.getStableInsetRight();
                    ib = wi.getStableInsetBottom();
                    DisplayCutout c = wi.getDisplayCutout();
                    if (c != null) {
                        il = Math.max(il, c.getSafeInsetLeft());
                        it = Math.max(it, c.getSafeInsetTop());
                        ir = Math.max(ir, c.getSafeInsetRight());
                        ib = Math.max(ib, c.getSafeInsetBottom());
                    }
                }
            }
            // No insets yet (window not attached) or the system gave zeros: use the bar sizes from resources.
            if (it == 0) {
                it = barSize("status_bar_height");
            }
            if (ib == 0 && il == 0 && ir == 0) {
                ib = barSize("navigation_bar_height");
            }
            // NativeActivity draws on the window surface itself, which covers the whole screen whatever the decor
            // view does, so the full bar sizes overlap us. With the keyboard up the window is already resized.
            if (Build.VERSION.SDK_INT >= 30 && wi != null && wi.isVisible(WindowInsets.Type.ime())) {
                ib = 0;
            }
            l = il;
            t = it;
            r = ir;
            b = ib;
        }
        boolean dark = isNight();
        String key = dark + ":" + l + ":" + t + ":" + r + ":" + b;
        if (!force && key.equals(lastEnv)) {
            return;
        }
        lastEnv = key;
        try {
            nativeEnv(dark, l, t, r, b);
        } catch (UnsatisfiedLinkError e) {
            // Rust has not registered the method yet: try again on the next layout.
            lastEnv = "";
        }
    }

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
