package com.pdfoxide.util;

/**
 * Detects supported features in pdf_oxide.
 */
public final class FeatureDetection {
    private static volatile Boolean supportsTaggedPdf = null;
    private static volatile Boolean supportsXfa = null;
    private static volatile Boolean supportsAcroForms = null;

    private FeatureDetection() {
    }

    /**
     * Checks if Tagged PDF (PDF/UA) is supported.
     *
     * @return true if supported
     */
    public static boolean supportsTaggedPdf() {
        if (supportsTaggedPdf == null) {
            supportsTaggedPdf = nativeSupportsTaggedPdf();
        }
        return supportsTaggedPdf;
    }

    /**
     * Checks if XFA forms are supported.
     *
     * @return true if supported
     */
    public static boolean supportsXfa() {
        if (supportsXfa == null) {
            supportsXfa = nativeSupportsXfa();
        }
        return supportsXfa;
    }

    /**
     * Checks if AcroForms are supported.
     *
     * @return true if supported
     */
    public static boolean supportsAcroForms() {
        if (supportsAcroForms == null) {
            supportsAcroForms = nativeSupportsAcroForms();
        }
        return supportsAcroForms;
    }

    private static native boolean nativeSupportsTaggedPdf();
    private static native boolean nativeSupportsXfa();
    private static native boolean nativeSupportsAcroForms();
}
