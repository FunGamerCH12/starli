package dev.horizon.android;

/** SAF result callback and storage permission only. The native activity, UI and file operations stay Rust. */
public final class HorizonActivity extends android.app.NativeActivity {
    static { System.loadLibrary("horizon_android"); }
    private static final int STORAGE_REQUEST = 4102;
    private final Object storageLock = new Object();
    private boolean storageAnswered;
    private native void nativeDocumentResult(int request, int result, String uri);

    @Override protected void onActivityResult(int request, int result, android.content.Intent data) {
        super.onActivityResult(request, result, data);
        if (request == STORAGE_REQUEST) { storageDone(); return; }
        if (request != 4101) return;
        nativeDocumentResult(request, result,
                data == null || data.getData() == null ? null : data.getData().toString());
    }

    @Override public void onRequestPermissionsResult(int request, String[] permissions, int[] results) {
        super.onRequestPermissionsResult(request, permissions, results);
        if (request == STORAGE_REQUEST) storageDone();
    }

    private void storageDone() {
        synchronized (storageLock) { storageAnswered = true; storageLock.notifyAll(); }
    }

    private void askStorage() {
        try {
            if (android.os.Build.VERSION.SDK_INT >= 30) {
                android.content.Intent intent = new android.content.Intent(
                        android.provider.Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                        android.net.Uri.parse("package:" + getPackageName()));
                startActivityForResult(intent, STORAGE_REQUEST);
            } else {
                requestPermissions(new String[] {
                        android.Manifest.permission.WRITE_EXTERNAL_STORAGE,
                        android.Manifest.permission.READ_EXTERNAL_STORAGE }, STORAGE_REQUEST);
            }
        } catch (RuntimeException error) {
            storageDone();
        }
    }

    /** Called from the native thread. Returns /storage/emulated/0/MC, or null if unavailable. */
    public String awaitStorage() {
        if (!HorizonStorage.granted(this)) {
            synchronized (storageLock) { storageAnswered = false; }
            runOnUiThread(this::askStorage);
            synchronized (storageLock) {
                long end = System.currentTimeMillis() + 10 * 60 * 1000L;
                while (!storageAnswered && System.currentTimeMillis() < end) {
                    try { storageLock.wait(1000); }
                    catch (InterruptedException error) { Thread.currentThread().interrupt(); break; }
                }
            }
        }
        return HorizonStorage.shared(this);
    }
}
