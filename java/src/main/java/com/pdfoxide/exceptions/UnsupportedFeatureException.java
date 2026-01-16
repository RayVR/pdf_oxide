package com.pdfoxide.exceptions;
public class UnsupportedFeatureException extends PdfException {
    public UnsupportedFeatureException(String feature) {
        super("Feature not available: " + feature + ". Rebuild with appropriate feature flag.");
    }
}
