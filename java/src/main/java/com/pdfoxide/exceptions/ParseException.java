package com.pdfoxide.exceptions;

/**
 * Exception thrown when PDF parsing or structure errors occur.
 *
 * <p>This includes errors such as invalid PDF format, unexpected tokens,
 * corrupted PDF structure, and other parsing-related issues.
 *
 * @since 1.0.0
 */
public class ParseException extends PdfException {
    /**
     * Constructs a new ParseException with the specified detail message.
     *
     * @param message the detail message
     */
    public ParseException(String message) {
        super(message);
    }

    /**
     * Constructs a new ParseException with the specified detail message and cause.
     *
     * @param message the detail message
     * @param cause the cause
     */
    public ParseException(String message, Throwable cause) {
        super(message, cause);
    }
}
