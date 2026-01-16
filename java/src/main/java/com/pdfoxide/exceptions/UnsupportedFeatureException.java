package com.pdfoxide.exceptions;

/**
 * Exception thrown when a feature is not available.
 *
 * <p>This occurs when attempting to use optional features (like OCR, rendering,
 * or digital signatures) that were not compiled into the native library.
 *
 * @since 1.0.0
 */
public class UnsupportedFeatureException extends PdfException {
    /**
     * Constructs a new UnsupportedFeatureException for the specified feature.
     *
     * @param feature the name of the missing feature
     */
    public UnsupportedFeatureException(String feature) {
        super(String.format(
            "Feature not available: %s. Rebuild with appropriate feature flag.",
            feature
        ));
    }

    /**
     * Constructs a new UnsupportedFeatureException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public UnsupportedFeatureException(String message, Throwable cause) {
        super(message, cause);
    }
}
