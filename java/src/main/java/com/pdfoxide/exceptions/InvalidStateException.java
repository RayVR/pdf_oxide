package com.pdfoxide.exceptions;

/**
 * Exception thrown when an invalid operation is attempted for the current document state.
 *
 * <p>For example, attempting to modify a closed document or accessing a page that
 * doesn't exist.
 *
 * @since 1.0.0
 */
public class InvalidStateException extends PdfException {
    /**
     * Constructs a new InvalidStateException with the specified detail message.
     *
     * @param message the detail message
     */
    public InvalidStateException(String message) {
        super(message);
    }

    /**
     * Constructs a new InvalidStateException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public InvalidStateException(String message, Throwable cause) {
        super(message, cause);
    }
}
