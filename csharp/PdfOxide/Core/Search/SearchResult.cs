using System;
using PdfOxide.Exceptions;
using PdfOxide.Geometry;
using PdfOxide.Internal;

namespace PdfOxide.Core.Search
{
    /// <summary>
    /// Represents a text search result on a PDF page.
    /// </summary>
    /// <remarks>
    /// <para>
    /// SearchResult represents a single occurrence of a search term found on a page.
    /// It provides access to the text content, bounding box location, and page number.
    /// </para>
    /// </remarks>
    /// <example>
    /// <code>
    /// var page = doc.GetPage(0);
    /// var results = page.FindText("hello");
    /// 
    /// foreach (var result in results)
    /// {
    ///     Console.WriteLine($"Found: {result.Text}");
    ///     Console.WriteLine($"At: {result.BoundingBox}");
    ///     Console.WriteLine($"Page: {result.PageIndex}");
    /// }
    /// </code>
    /// </example>
    public sealed class SearchResult : IDisposable
    {
        private NativeHandle _handle;
        private bool _disposed;

        /// <summary>
        /// Gets the matched text content.
        /// </summary>
        /// <value>The text that was found.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the result has been disposed.</exception>
        public string Text
        {
            get
            {
                ThrowIfDisposed();
                var ptr = NativeMethods.PdfSearchResultGetText(_handle.DangerousGetHandle(),
                    out var errorCode);
                ExceptionMapper.ThrowIfError(errorCode);

                if (ptr == IntPtr.Zero)
                    return string.Empty;

                try
                {
                    return StringMarshaler.PtrToString(ptr);
                }
                finally
                {
                    NativeMethods.FreeString(ptr);
                }
            }
        }

        /// <summary>
        /// Gets the bounding box of the search result on the page.
        /// </summary>
        /// <value>The rectangle containing the matched text.</value>
        public Rect BoundingBox
        {
            get
            {
                ThrowIfDisposed();
                NativeMethods.PdfSearchResultGetBbox(_handle.DangerousGetHandle(),
                    out var x, out var y, out var width, out var height);
                return new Rect(x, y, width, height);
            }
        }

        /// <summary>
        /// Gets the page index where this result was found.
        /// </summary>
        /// <value>The page index (0-based).</value>
        public int PageIndex
        {
            get
            {
                ThrowIfDisposed();
                var page = NativeMethods.PdfSearchResultGetPage(_handle.DangerousGetHandle(),
                    out var errorCode);
                ExceptionMapper.ThrowIfError(errorCode);
                return page;
            }
        }

        /// <summary>
        /// Gets the left coordinate of the result in points.
        /// </summary>
        public float Left => BoundingBox.X;

        /// <summary>
        /// Gets the top coordinate of the result in points.
        /// </summary>
        public float Top => BoundingBox.Y;

        /// <summary>
        /// Gets the width of the result in points.
        /// </summary>
        public float Width => BoundingBox.Width;

        /// <summary>
        /// Gets the height of the result in points.
        /// </summary>
        public float Height => BoundingBox.Height;

        /// <summary>
        /// Gets the center point of the result.
        /// </summary>
        public Point Center => new Point(
            BoundingBox.X + BoundingBox.Width / 2,
            BoundingBox.Y + BoundingBox.Height / 2);

        internal SearchResult(NativeHandle handle)
        {
            _handle = handle ?? throw new ArgumentNullException(nameof(handle));
        }

        /// <summary>
        /// Disposes the search result and releases native resources.
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
                throw new ObjectDisposedException(nameof(SearchResult));
        }
    }
}
