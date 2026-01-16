using System;

namespace PdfOxide.Exceptions
{
    /// <summary>
    /// Base exception for all PDF processing errors.
    /// </summary>
    /// <remarks>
    /// Thrown when a generic or unknown PDF error occurs.
    /// </remarks>
    public class PdfException : Exception
    {
        /// <summary>
        /// Gets the native error code from the Rust FFI layer.
        /// </summary>
        public int ErrorCode { get; }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfException"/> class.
        /// </summary>
        /// <param name="message">The error message.</param>
        public PdfException(string message) : base(message)
        {
            ErrorCode = 100;
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfException"/> class with an error code.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="errorCode">The native error code.</param>
        public PdfException(string message, int errorCode) : base(message)
        {
            ErrorCode = errorCode;
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfException"/> class with an inner exception.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public PdfException(string message, Exception innerException)
            : base(message, innerException)
        {
            ErrorCode = 100;
        }
    }

    /// <summary>
    /// Thrown when an I/O error occurs (file not found, permission denied, etc.).
    /// </summary>
    public class PdfIoException : PdfException
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="PdfIoException"/> class.
        /// </summary>
        /// <param name="message">The error message.</param>
        public PdfIoException(string message) : base(message, 1) { }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfIoException"/> class with an inner exception.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public PdfIoException(string message, Exception innerException)
            : base(message, innerException) { }
    }

    /// <summary>
    /// Thrown when a PDF structure parsing error occurs.
    /// </summary>
    public class PdfParseException : PdfException
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="PdfParseException"/> class.
        /// </summary>
        /// <param name="message">The error message.</param>
        public PdfParseException(string message) : base(message, 2) { }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfParseException"/> class with an inner exception.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public PdfParseException(string message, Exception innerException)
            : base(message, innerException) { }
    }

    /// <summary>
    /// Thrown when an encryption or password error occurs.
    /// </summary>
    public class PdfEncryptionException : PdfException
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="PdfEncryptionException"/> class.
        /// </summary>
        /// <param name="message">The error message.</param>
        public PdfEncryptionException(string message) : base(message, 3) { }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfEncryptionException"/> class with an inner exception.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public PdfEncryptionException(string message, Exception innerException)
            : base(message, innerException) { }
    }

    /// <summary>
    /// Thrown when an operation is not allowed in the current document state.
    /// </summary>
    public class PdfInvalidStateException : PdfException
    {
        /// <summary>
        /// Initializes a new instance of the <see cref="PdfInvalidStateException"/> class.
        /// </summary>
        /// <param name="message">The error message.</param>
        public PdfInvalidStateException(string message) : base(message, 4) { }

        /// <summary>
        /// Initializes a new instance of the <see cref="PdfInvalidStateException"/> class with an inner exception.
        /// </summary>
        /// <param name="message">The error message.</param>
        /// <param name="innerException">The inner exception.</param>
        public PdfInvalidStateException(string message, Exception innerException)
            : base(message, innerException) { }
    }

    /// <summary>
    /// Thrown when a requested feature is not available or not enabled.
    /// </summary>
    public class UnsupportedFeatureException : PdfException
    {
        /// <summary>
        /// Gets the name of the unsupported feature.
        /// </summary>
        public string FeatureName { get; }

        /// <summary>
        /// Initializes a new instance of the <see cref="UnsupportedFeatureException"/> class.
        /// </summary>
        /// <param name="featureName">The name of the unsupported feature.</param>
        public UnsupportedFeatureException(string featureName)
            : base($"Unsupported feature: {featureName}", GetErrorCode(featureName))
        {
            FeatureName = featureName;
        }

        private static int GetErrorCode(string featureName) => featureName switch
        {
            "rendering" => 5,
            "ocr" => 6,
            _ => 100
        };
    }
}
