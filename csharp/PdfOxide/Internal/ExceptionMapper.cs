using PdfOxide.Exceptions;

namespace PdfOxide.Internal
{
    /// <summary>
    /// Maps native error codes to .NET exceptions.
    /// </summary>
    internal static class ExceptionMapper
    {
        /// <summary>
        /// Creates an exception from an error code.
        /// </summary>
        /// <param name="errorCode">The error code from the Rust FFI layer.</param>
        /// <returns>An appropriate <see cref="PdfException"/> subclass.</returns>
        public static PdfException CreateException(int errorCode)
        {
            return errorCode switch
            {
                0 => new PdfException("Success (no error)"),
                1 => new PdfIoException("I/O error: File not found, permission denied, or read/write failed"),
                2 => new PdfParseException("Parse error: Invalid PDF structure or content stream"),
                3 => new PdfEncryptionException("Encryption error: Incorrect password or unsupported encryption"),
                4 => new PdfInvalidStateException("Invalid state: Operation not allowed in current document state"),
                5 => new UnsupportedFeatureException("rendering"),
                6 => new UnsupportedFeatureException("ocr"),
                _ => new PdfException($"Unknown error (code: {errorCode})")
            };
        }

        /// <summary>
        /// Checks if an error code represents success.
        /// </summary>
        /// <param name="errorCode">The error code.</param>
        /// <returns>True if the error code indicates success (0), false otherwise.</returns>
        public static bool IsSuccess(int errorCode) => errorCode == 0;

        /// <summary>
        /// Throws an exception if the error code indicates an error.
        /// </summary>
        /// <param name="errorCode">The error code to check.</param>
        /// <exception cref="PdfException">Thrown if the error code indicates an error.</exception>
        public static void ThrowIfError(int errorCode)
        {
            if (!IsSuccess(errorCode))
            {
                throw CreateException(errorCode);
            }
        }
    }
}
