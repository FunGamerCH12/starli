package dev.horizon.android;

import android.content.Context;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Environment;
import java.io.File;

/** Where app data lives: /storage/emulated/0/MC when permitted, else app-private files. */
final class HorizonStorage {
    static final String FOLDER = "MC";
    private HorizonStorage() {}

    static boolean granted(Context context) {
        if (Build.VERSION.SDK_INT >= 30) return Environment.isExternalStorageManager();
        if (Build.VERSION.SDK_INT >= 23) {
            return context.checkSelfPermission(android.Manifest.permission.WRITE_EXTERNAL_STORAGE)
                    == PackageManager.PERMISSION_GRANTED;
        }
        return true;
    }

    /** The shared folder if access is granted and the folder is writable, else null. */
    static String shared(Context context) {
        if (!granted(context)) return null;
        File dir = new File(Environment.getExternalStorageDirectory(), FOLDER);
        if ((dir.isDirectory() || dir.mkdirs()) && dir.canWrite()) return dir.getAbsolutePath();
        return null;
    }

    /** Never blocks; matches what the native side chose. */
    static String resolve(Context context) {
        String shared = shared(context);
        return shared != null ? shared : context.getFilesDir().getAbsolutePath();
    }
}
