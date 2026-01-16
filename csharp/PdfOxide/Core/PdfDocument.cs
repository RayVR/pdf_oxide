using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using PdfOxide.Exceptions;
using PdfOxide.Internal;

namespace PdfOxide.Core
{
    /// <summary>
    /// Represents a PDF document for reading and text extraction.
    /// Provides read-only access with automatic reading order detection.
    /// </summary>
    /// <remarks>
    /// <para>
    /// PdfDocument is the primary API for opening and reading existing PDF files.
    /// It supports:
    /// <list type="bullet">
    /// <item><description>Opening PDF files from disk or memory</description></item>
    /// <item><description>Extracting text with automatic reading order detection</description></item>
    /// <item><description>Converting pages to various formats (Markdown, HTML, PlainText)</description></item>
    /// <item><description>Accessing PDF metadata and structure information</description></item>
    /// </list>
    /// </para>
    /// <para>
    /// The document must be explicitly disposed to release native resources.
    /// Use 'using' statements for automatic cleanup.
    /// </para>
    /// </remarks>
    /// <example>
    /// <code>
    /// using (var doc = PdfDocument.Open("document.pdf"))
    /// {
    ///     // Get PDF version and page count
    ///     var version = doc.Version;
    ///     var pageCount = doc.PageCount;
    ///     Console.WriteLine($"PDF {version.Major}.{version.Minor}, {pageCount} pages");
    ///
    ///     // Extract text from first page
    ///     var text = doc.ExtractText(0);
    ///     Console.WriteLine(text);
    ///
    ///     // Convert to Markdown
    ///     var markdown = doc.ToMarkdown(0);
    ///     File.WriteAllText("output.md", markdown);
    /// }
    /// </code>
    /// </example>
    public sealed class PdfDocument : IDisposable
    {
        private NativeHandle _handle;
        private bool _disposed;

        private PdfDocument(NativeHandle handle)
        {
            _handle = handle ?? throw new ArgumentNullException(nameof(handle));
        }

        /// <summary>
        /// Opens a PDF document from a file path.
        /// </summary>
        /// <param name="path">The file path to the PDF.</param>
        /// <returns>An opened PdfDocument.</returns>
        /// <exception cref="ArgumentNullException">Thrown if <paramref name="path"/> is null.</exception>
        /// <exception cref="PdfIoException">Thrown if the file cannot be opened.</exception>
        /// <exception cref="PdfParseException">Thrown if the PDF is invalid.</exception>
        public static PdfDocument Open(string path)
        {
            if (path == null)
                throw new ArgumentNullException(nameof(path));

            var handle = NativeMethods.PdfDocumentOpen(path, out var errorCode);
            if (handle.IsInvalid)
            {
                ExceptionMapper.ThrowIfError(errorCode);
            }

            return new PdfDocument(handle);
        }

        /// <summary>
        /// Opens a PDF document from a stream.
        /// </summary>
        /// <param name="stream">The stream containing PDF data.</param>
        /// <returns>An opened PdfDocument.</returns>
        /// <exception cref="ArgumentNullException">Thrown if <paramref name="stream"/> is null.</exception>
        /// <exception cref="PdfIoException">Thrown if the stream cannot be read.</exception>
        /// <exception cref="PdfParseException">Thrown if the PDF is invalid.</exception>
        public static PdfDocument Open(Stream stream)
        {
            if (stream == null)
                throw new ArgumentNullException(nameof(stream));

            byte[] data;
            using (var ms = new MemoryStream())
            {
                stream.CopyTo(ms);
                data = ms.ToArray();
            }

            var handle = NativeMethods.PdfDocumentOpenFromBytes(data, data.Length, out var errorCode);
            if (handle.IsInvalid)
            {
                ExceptionMapper.ThrowIfError(errorCode);
            }

            return new PdfDocument(handle);
        }

        /// <summary>
        /// Gets the PDF version as a tuple of (major, minor).
        /// </summary>
        /// <value>A tuple containing the major and minor version numbers.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        public (byte Major, byte Minor) Version
        {
            get
            {
                ThrowIfDisposed();
                NativeMethods.PdfDocumentGetVersion(_handle, out var major, out var minor);
                return (major, minor);
            }
        }

        /// <summary>
        /// Gets the number of pages in the document.
        /// </summary>
        /// <value>The page count.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if page count cannot be determined.</exception>
        public int PageCount
        {
            get
            {
                ThrowIfDisposed();
                var count = NativeMethods.PdfDocumentGetPageCount(_handle, out var errorCode);
                ExceptionMapper.ThrowIfError(errorCode);
                return count;
            }
        }

        /// <summary>
        /// Gets a value indicating whether the document has a structure tree (Tagged PDF).
        /// </summary>
        /// <value>True if the document has a structure tree, false otherwise.</value>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        public bool HasStructureTree
        {
            get
            {
                ThrowIfDisposed();
                return NativeMethods.PdfDocumentHasStructureTree(_handle);
            }
        }

        /// <summary>
        /// Extracts text from a page with automatic reading order detection.
        /// </summary>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <returns>The extracted text.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if <paramref name="pageIndex"/> is out of range.</exception>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if text extraction fails.</exception>
        public string ExtractText(int pageIndex)
        {
            ThrowIfDisposed();

            if (pageIndex < 0 || pageIndex >= PageCount)
                throw new ArgumentOutOfRangeException(nameof(pageIndex));

            var ptr = NativeMethods.PdfDocumentExtractText(_handle, pageIndex, out var errorCode);
            ExceptionMapper.ThrowIfError(errorCode);

            return StringMarshaler.PtrToStringAndFree(ptr);
        }

        /// <summary>
        /// Asynchronously extracts text from a page.
        /// </summary>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <param name="cancellationToken">A cancellation token.</param>
        /// <returns>A task that yields the extracted text.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if <paramref name="pageIndex"/> is out of range.</exception>
        /// <exception cref="OperationCanceledException">Thrown if the operation is cancelled.</exception>
        public Task<string> ExtractTextAsync(int pageIndex, CancellationToken cancellationToken = default)
        {
            return Task.Run(() =>
            {
                cancellationToken.ThrowIfCancellationRequested();
                return ExtractText(pageIndex);
            }, cancellationToken);
        }

        /// <summary>
        /// Converts a page to Markdown format.
        /// </summary>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <returns>The page content as Markdown.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if <paramref name="pageIndex"/> is out of range.</exception>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if conversion fails.</exception>
        public string ToMarkdown(int pageIndex)
        {
            ThrowIfDisposed();

            if (pageIndex < 0 || pageIndex >= PageCount)
                throw new ArgumentOutOfRangeException(nameof(pageIndex));

            var ptr = NativeMethods.PdfDocumentToMarkdown(_handle, pageIndex, out var errorCode);
            ExceptionMapper.ThrowIfError(errorCode);

            return StringMarshaler.PtrToStringAndFree(ptr);
        }

        /// <summary>
        /// Converts all pages to Markdown format.
        /// </summary>
        /// <returns>The document content as Markdown.</returns>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if conversion fails.</exception>
        public string ToMarkdownAll()
        {
            ThrowIfDisposed();

            var ptr = NativeMethods.PdfDocumentToMarkdownAll(_handle, out var errorCode);
            ExceptionMapper.ThrowIfError(errorCode);

            return StringMarshaler.PtrToStringAndFree(ptr);
        }

        /// <summary>
        /// Converts a page to HTML format.
        /// </summary>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <returns>The page content as HTML.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if <paramref name="pageIndex"/> is out of range.</exception>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if conversion fails.</exception>
        public string ToHtml(int pageIndex)
        {
            ThrowIfDisposed();

            if (pageIndex < 0 || pageIndex >= PageCount)
                throw new ArgumentOutOfRangeException(nameof(pageIndex));

            var ptr = NativeMethods.PdfDocumentToHtml(_handle, pageIndex, out var errorCode);
            ExceptionMapper.ThrowIfError(errorCode);

            return StringMarshaler.PtrToStringAndFree(ptr);
        }

        /// <summary>
        /// Converts a page to plain text format.
        /// </summary>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <returns>The page content as plain text.</returns>
        /// <exception cref="ArgumentOutOfRangeException">Thrown if <paramref name="pageIndex"/> is out of range.</exception>
        /// <exception cref="ObjectDisposedException">Thrown if the document has been disposed.</exception>
        /// <exception cref="PdfException">Thrown if conversion fails.</exception>
        public string ToPlainText(int pageIndex)
        {
            ThrowIfDisposed();

            if (pageIndex < 0 || pageIndex >= PageCount)
                throw new ArgumentOutOfRangeException(nameof(pageIndex));

            var ptr = NativeMethods.PdfDocumentToPlainText(_handle, pageIndex, out var errorCode);
            ExceptionMapper.ThrowIfError(errorCode);

            return StringMarshaler.PtrToStringAndFree(ptr);
        }

        /// <summary>
        /// Disposes the document and releases native resources.
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
                throw new ObjectDisposedException(nameof(PdfDocument));
        }
    }
}
