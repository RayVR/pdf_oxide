package com.pdfoxide.exceptions;

/**
 * Exception thrown for encryption or password-related errors.
 *
 * <p>This includes errors such as incorrect password, encryption/decryption failures,
 * and unsupported encryption algorithms.
 *
 * @since 1.0.0
 */
public class EncryptionException extends PdfException {
    /**
     * Constructs a new EncryptionException with the specified detail message.
     *
     * @param message the detail message
     */
    public EncryptionException(String message) {
        super(message);
    }

    /**
     * Constructs a new EncryptionException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public EncryptionException(String message, Throwable cause) {
        super(message, cause);
    }
}
