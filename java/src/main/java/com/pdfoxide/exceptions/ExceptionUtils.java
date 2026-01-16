package com.pdfoxide.exceptions;

/**
 * Utility methods for exception handling.
 */
public final class ExceptionUtils {
    private ExceptionUtils() {
    }

    /**
     * Throws appropriate exception for error code.
     *
     * @param errorCode error code from native
     * @param message error message
     * @throws PdfException always
     */
    public static void throwException(int errorCode, String message) throws PdfException {
        switch (errorCode) {
            case 1:
                throw new ParseException(message);
            case 2:
                throw new EncryptionException(message);
            case 3:
                throw new IoException(message);
            case 4:
                throw new InvalidStateException(message);
            case 5:
                throw new UnsupportedFeatureException(message);
            default:
                throw new PdfException(message);
        }
    }
}
