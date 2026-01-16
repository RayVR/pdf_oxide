using System;
using System.Runtime.InteropServices;

namespace PdfOxide.Internal
{
    /// <summary>
    /// P/Invoke declarations for the native pdf_oxide library.
    /// </summary>
    /// <remarks>
    /// This class declares all the FFI functions exported from the Rust library.
    /// All functions are blittable and use standard calling conventions for maximum compatibility.
    /// </remarks>
    internal static class NativeMethods
    {
        private const string LibName = "pdf_oxide";
        private const CallingConvention DefaultCallingConvention = CallingConvention.Cdecl;

        #region PdfDocument API

        /// <summary>
        /// Opens a PDF document from a file path.
        /// </summary>
        /// <param name="path">UTF-8 null-terminated file path.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to the PDF document, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern NativeHandle PdfDocumentOpen(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            out int errorCode);

        /// <summary>
        /// Opens a PDF document from a memory buffer.
        /// </summary>
        /// <param name="data">Pointer to PDF bytes.</param>
        /// <param name="length">Length of the data buffer.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to the PDF document, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern NativeHandle PdfDocumentOpenFromBytes(
            [In] byte[] data,
            int length,
            out int errorCode);

        /// <summary>
        /// Frees a PdfDocument handle.
        /// </summary>
        /// <param name="handle">The handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfDocumentFree(IntPtr handle);

        /// <summary>
        /// Gets the PDF version.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="major">Output parameter for major version number.</param>
        /// <param name="minor">Output parameter for minor version number.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfDocumentGetVersion(
            NativeHandle handle,
            out byte major,
            out byte minor);

        /// <summary>
        /// Gets the number of pages in the document.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The page count, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfDocumentGetPageCount(
            NativeHandle handle,
            out int errorCode);

        /// <summary>
        /// Checks if the document has a structure tree (Tagged PDF).
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <returns>True if the document has a structure tree, false otherwise.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        [return: MarshalAs(UnmanagedType.I1)]
        public static extern bool PdfDocumentHasStructureTree(NativeHandle handle);

        /// <summary>
        /// Extracts text from a page.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfDocumentExtractText(
            NativeHandle handle,
            int pageIndex,
            out int errorCode);

        /// <summary>
        /// Converts a page to Markdown format.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfDocumentToMarkdown(
            NativeHandle handle,
            int pageIndex,
            out int errorCode);

        /// <summary>
        /// Converts all pages to Markdown format.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfDocumentToMarkdownAll(
            NativeHandle handle,
            out int errorCode);

        /// <summary>
        /// Converts a page to HTML format.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfDocumentToHtml(
            NativeHandle handle,
            int pageIndex,
            out int errorCode);

        /// <summary>
        /// Converts a page to plain text format.
        /// </summary>
        /// <param name="handle">The document handle.</param>
        /// <param name="pageIndex">The page index (0-based).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfDocumentToPlainText(
            NativeHandle handle,
            int pageIndex,
            out int errorCode);

        #endregion

        #region Memory Management

        /// <summary>
        /// Frees a UTF-8 string allocated by Rust.
        /// </summary>
        /// <param name="ptr">Pointer to the string to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void FreeString(IntPtr ptr);

        /// <summary>
        /// Frees a byte buffer allocated by Rust.
        /// </summary>
        /// <param name="ptr">Pointer to the buffer to free.</param>
        /// <param name="len">Length of the buffer (for validation).</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void FreeBytes(IntPtr ptr, int len);

        #endregion

        #region Utility Functions

        /// <summary>
        /// Allocates a string in Rust memory.
        /// </summary>
        /// <param name="s">UTF-8 null-terminated string pointer.</param>
        /// <returns>Allocated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern IntPtr AllocString(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string s);

        #endregion
    }
}
