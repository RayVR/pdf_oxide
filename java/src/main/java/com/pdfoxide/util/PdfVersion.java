package com.pdfoxide.util;

/**
 * PDF version information.
 */
public final class PdfVersion {
    private final int major;
    private final int minor;

    public PdfVersion(int major, int minor) {
        this.major = major;
        this.minor = minor;
    }

    public int getMajor() {
        return major;
    }

    public int getMinor() {
        return minor;
    }

    @Override
    public String toString() {
        return major + "." + minor;
    }
}
