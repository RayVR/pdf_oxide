package com.pdfoxide.exceptions;

/**
 * Base checked exception for all PDF operations.
 *
 * <p>All exceptions in the pdf_oxide library inherit from this base exception,
 * allowing for uniform error handling across the API.
 *
 * @since 1.0.0
 */
public class PdfException extends Exception {
    /**
     * Constructs a new PdfException with the specified detail message.
     *
     * @param message the detail message
     */
    public PdfException(String message) {
        super(message);
    }

    /**
     * Constructs a new PdfException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public PdfException(String message, Throwable cause) {
        super(message, cause);
    }
}
