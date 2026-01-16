package com.pdfoxide.util;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Loads the native pdf_oxide JNI library.
 */
public final class NativeLibraryLoader {
    private static volatile boolean loaded = false;
    private static final String LIB_NAME = "pdf_oxide_jni";

    private NativeLibraryLoader() {
    }

    /**
     * Loads the native library.
     *
     * @throws Exception if library cannot be loaded
     */
    public static synchronized void load() throws Exception {
        if (loaded) {
            return;
        }

        try {
            System.loadLibrary(LIB_NAME);
            loaded = true;
        } catch (UnsatisfiedLinkError e) {
            // Try loading from resources
            loadFromResources();
            loaded = true;
        }
    }

    private static void loadFromResources() throws Exception {
        String osName = System.getProperty("os.name").toLowerCase();
        String osArch = System.getProperty("os.arch").toLowerCase();
        String libPath = getLibraryPath(osName, osArch);

        try (InputStream in = NativeLibraryLoader.class.getResourceAsStream(libPath)) {
            if (in == null) {
                throw new UnsatisfiedLinkError("Native library not found: " + libPath);
            }

            // Extract to temp file
            Path tempFile = Files.createTempFile("pdf_oxide_jni", getLibExtension(osName));
            tempFile.toFile().deleteOnExit();
            Files.copy(in, tempFile, java.nio.file.StandardCopyOption.REPLACE_EXISTING);

            System.load(tempFile.toString());
        }
    }

    private static String getLibraryPath(String osName, String osArch) {
        String arch = mapArch(osArch);
        if (osName.contains("win")) {
            return "/natives/windows-" + arch + "/pdf_oxide_jni.dll";
        } else if (osName.contains("mac")) {
            return "/natives/macos-" + arch + "/libpdf_oxide_jni.dylib";
        } else {
            return "/natives/linux-" + arch + "/libpdf_oxide_jni.so";
        }
    }

    private static String mapArch(String osArch) {
        if (osArch.contains("64")) {
            return osArch.contains("aarch64") || osArch.contains("arm64") ? "aarch64" : "x86_64";
        }
        return osArch;
    }

    private static String getLibExtension(String osName) {
        if (osName.contains("win")) {
            return ".dll";
        } else if (osName.contains("mac")) {
            return ".dylib";
        } else {
            return ".so";
        }
    }
}
