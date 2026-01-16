using System;
using PdfOxide.Internal;

namespace PdfOxide.Core
{
    /// <summary>
    /// Represents a page within a PDF document.
    /// Provides access to page properties and content elements.
    /// </summary>
    /// <remarks>
    /// <para>
    /// PdfPage provides DOM-like access to page content with:
    /// <list type="bullet">
    /// <item><description>Page dimensions and properties</description></item>
    /// <item><description>Element access and manipulation</description></item>
    /// <item><description>Geometric transformations</description></item>
    /// </list>
    /// </para>
    /// <para>
    /// The page must be explicitly disposed to release native resources.
    /// Use 'using' statements for automatic cleanup.
    /// </para>
    /// </remarks>
    /// <example>
    /// <code>
    /// using (var doc = PdfDocument.Open("document.pdf"))
    /// {
    ///     for (int i = 0; i < doc.PageCount; i++)
    ///     {
    ///         var page = doc.GetPage(i);
    ///         Console.WriteLine($"Page {i}: {page.Width}x{page.Height}");
    ///     }
    /// }
    /// </code>
    /// </example>
    public sealed class PdfPage : IDisposable
    {
        private NativeHandle _handle;
        private bool _disposed;

        internal PdfPage(NativeHandle handle)
        {
            _handle = handle ?? throw new ArgumentNullException(nameof(handle));
        }

        /// <summary>
        /// Gets the width of the page in points.
        /// </summary>
        /// <value>The page width.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        public float Width
        {
            get
            {
                ThrowIfDisposed();
                return NativeMethods.PdfPageGetWidth(_handle.DangerousGetHandle());
            }
        }

        /// <summary>
        /// Gets the height of the page in points.
        /// </summary>
        /// <value>The page height.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        public float Height
        {
            get
            {
                ThrowIfDisposed();
                return NativeMethods.PdfPageGetHeight(_handle.DangerousGetHandle());
            }
        }

        /// <summary>
        /// Gets the zero-based page index.
        /// </summary>
        /// <value>The page index.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        public int Index
        {
            get
            {
                ThrowIfDisposed();
                return NativeMethods.PdfPageGetIndex(_handle.DangerousGetHandle());
            }
        }

        /// <summary>
        /// Gets the page dimensions as a tuple (width, height) in points.
        /// </summary>
        /// <value>A tuple containing width and height.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        /// <example>
        /// <code>
        /// var (width, height) = page.Dimensions;
        /// Console.WriteLine($"Page size: {width}x{height}");
        /// </code>
        /// </example>
        public (float Width, float Height) Dimensions
        {
            get
            {
                ThrowIfDisposed();
                NativeMethods.PdfPageGetDimensions(_handle.DangerousGetHandle(),
                    out var width, out var height);
                return (width, height);
            }
        }

        /// <summary>
        /// Gets the aspect ratio of the page (width / height).
        /// </summary>
        /// <value>The page aspect ratio.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        public float AspectRatio
        {
            get
            {
                ThrowIfDisposed();
                float height = Height;
                if (Math.Abs(height) < 0.001f)
                    return 0;
                return Width / height;
            }
        }

        /// <summary>
        /// Gets the area of the page in square points.
        /// </summary>
        /// <value>The page area.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the page has been disposed.</exception>
        public float Area
        {
            get
            {
                ThrowIfDisposed();
                return Width * Height;
            }
        }

        /// <summary>
        /// Disposes the page and releases native resources.
        /// </summary>
        public void Dispose()
        {
            if (!_disposed)
            {
                _handle?.Dispose();
                _disposed = true;
            }
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(PdfPage));
        }
    }
}
