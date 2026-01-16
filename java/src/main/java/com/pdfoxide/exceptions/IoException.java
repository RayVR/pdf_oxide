package com.pdfoxide.exceptions;

/**
 * Exception thrown for I/O errors when reading or writing PDF files.
 *
 * <p>This includes errors such as file not found, permission denied, and other
 * I/O related errors from the underlying filesystem.
 *
 * @since 1.0.0
 */
public class IoException extends PdfException {
    /**
     * Constructs a new IoException with the specified detail message.
     *
     * @param message the detail message
     */
    public IoException(String message) {
        super(message);
    }

    /**
     * Constructs a new IoException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public IoException(String message, Throwable cause) {
        super(message, cause);
    }
}
