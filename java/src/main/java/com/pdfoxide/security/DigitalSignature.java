package com.pdfoxide.security;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;

/**
 * Digital signature support (foundation in v0.3.0).
 */
public final class DigitalSignature {
    private final String name;
    private final String reason;
    private final String location;
    private final String date;

    public DigitalSignature(String name, String reason, String location, String date) {
        this.name = name;
        this.reason = reason;
        this.location = location;
        this.date = date;
    }

    /**
     * Gets the signature count in a document.
     *
     * @param document PDF document
     * @return number of signatures
     * @throws PdfException if operation fails
     */
    public static int getSignatureCount(PdfDocument document) throws PdfException {
        return nativeGetSignatureCount(document);
    }

    /**
     * Gets signature information.
     *
     * @param document PDF document
     * @param index signature index
     * @return signature information
     * @throws PdfException if operation fails
     */
    public static DigitalSignature getSignature(PdfDocument document, int index) throws PdfException {
        return nativeGetSignature(document, index);
    }

    public String getName() {
        return name;
    }

    public String getReason() {
        return reason;
    }

    public String getLocation() {
        return location;
    }

    public String getDate() {
        return date;
    }

    private static native int nativeGetSignatureCount(PdfDocument document) throws PdfException;
    private static native DigitalSignature nativeGetSignature(PdfDocument document, int index) throws PdfException;
}
