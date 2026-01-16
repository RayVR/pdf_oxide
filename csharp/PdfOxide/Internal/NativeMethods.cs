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

        #region Pdf Creation API

        /// <summary>
        /// Creates a PDF from Markdown content.
        /// </summary>
        /// <param name="markdown">UTF-8 null-terminated Markdown content.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to Pdf, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern NativeHandle PdfFromMarkdown(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string markdown,
            out int errorCode);

        /// <summary>
        /// Creates a PDF from HTML content.
        /// </summary>
        /// <param name="html">UTF-8 null-terminated HTML content.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to Pdf, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern NativeHandle PdfFromHtml(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string html,
            out int errorCode);

        /// <summary>
        /// Creates a PDF from plain text content.
        /// </summary>
        /// <param name="text">UTF-8 null-terminated text content.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to Pdf, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern NativeHandle PdfFromText(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string text,
            out int errorCode);

        /// <summary>
        /// Saves a PDF to file.
        /// </summary>
        /// <param name="handle">The PDF handle.</param>
        /// <param name="path">UTF-8 null-terminated output file path.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>0 on success, non-zero on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern int PdfSave(
            NativeHandle handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            out int errorCode);

        /// <summary>
        /// Saves a PDF to memory buffer.
        /// </summary>
        /// <param name="handle">The PDF handle.</param>
        /// <param name="outputPtr">Output parameter for byte buffer pointer.</param>
        /// <param name="outputLen">Output parameter for buffer size.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>0 on success, non-zero on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfSaveToBytes(
            NativeHandle handle,
            out IntPtr outputPtr,
            out ulong outputLen,
            out int errorCode);

        /// <summary>
        /// Gets the page count from a Pdf handle.
        /// </summary>
        /// <param name="handle">The PDF handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The page count, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfGetPageCount(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Frees a Pdf handle.
        /// </summary>
        /// <param name="handle">The handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfFree(IntPtr handle);

        #endregion

        #region DocumentEditor API

        /// <summary>
        /// Opens a PDF document for editing.
        /// </summary>
        /// <param name="path">UTF-8 null-terminated file path.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>Opaque handle to DocumentEditor, or null on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern NativeHandle DocumentEditorOpen(
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            out int errorCode);

        /// <summary>
        /// Frees a DocumentEditor handle.
        /// </summary>
        /// <param name="handle">The handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void DocumentEditorFree(IntPtr handle);

        /// <summary>
        /// Checks if the document has been modified.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <returns>True if modified, false otherwise.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        [return: MarshalAs(UnmanagedType.I1)]
        public static extern bool DocumentEditorIsModified(NativeHandle handle);

        /// <summary>
        /// Gets the source file path.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr DocumentEditorGetSourcePath(
            NativeHandle handle,
            out int errorCode);

        /// <summary>
        /// Gets the PDF version.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="major">Output parameter for major version number.</param>
        /// <param name="minor">Output parameter for minor version number.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void DocumentEditorGetVersion(
            NativeHandle handle,
            out byte major,
            out byte minor);

        /// <summary>
        /// Gets the number of pages in the document.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The page count, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int DocumentEditorGetPageCount(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the document title.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString), or null if not set.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr DocumentEditorGetTitle(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Sets the document title.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="title">UTF-8 null-terminated title string.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern void DocumentEditorSetTitle(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string title,
            out int errorCode);

        /// <summary>
        /// Gets the document author.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString), or null if not set.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr DocumentEditorGetAuthor(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Sets the document author.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="author">UTF-8 null-terminated author string.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern void DocumentEditorSetAuthor(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string author,
            out int errorCode);

        /// <summary>
        /// Gets the document subject.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer (must be freed with FreeString), or null if not set.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr DocumentEditorGetSubject(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Sets the document subject.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="subject">UTF-8 null-terminated subject string.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern void DocumentEditorSetSubject(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string subject,
            out int errorCode);

        /// <summary>
        /// Saves the document to a file.
        /// </summary>
        /// <param name="handle">The editor handle.</param>
        /// <param name="path">UTF-8 null-terminated output file path.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>0 on success, non-zero on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern int DocumentEditorSave(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            out int errorCode);

        #endregion

        #region DOM API

        /// <summary>
        /// Gets the width of a page.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <returns>The page width in points, or 0 if invalid.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern float PdfPageGetWidth(IntPtr handle);

        /// <summary>
        /// Gets the height of a page.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <returns>The page height in points, or 0 if invalid.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern float PdfPageGetHeight(IntPtr handle);

        /// <summary>
        /// Gets the page index.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <returns>The page index (0-based), or -1 if invalid.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfPageGetIndex(IntPtr handle);

        /// <summary>
        /// Gets the page dimensions.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <param name="widthOut">Output parameter for page width.</param>
        /// <param name="heightOut">Output parameter for page height.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfPageGetDimensions(
            IntPtr handle,
            out float widthOut,
            out float heightOut);

        /// <summary>
        /// Frees a PdfPage handle.
        /// </summary>
        /// <param name="handle">The handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfPageFree(IntPtr handle);

        #endregion

        #region Element API

        /// <summary>
        /// Gets the number of elements of a specific type on a page.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <param name="elementType">The type of element to count (ELEMENT_TYPE_*).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The number of elements found, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfPageFindElementsCount(
            IntPtr handle,
            int elementType,
            out int errorCode);

        /// <summary>
        /// Gets the text content of a text element.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfTextElementGetContent(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the font size of a text element.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The font size in points.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern float PdfTextElementGetFontSize(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the bounding box of an element.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <param name="x">Output parameter for x coordinate.</param>
        /// <param name="y">Output parameter for y coordinate.</param>
        /// <param name="width">Output parameter for width.</param>
        /// <param name="height">Output parameter for height.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfElementGetBbox(
            IntPtr handle,
            out float x,
            out float y,
            out float width,
            out float height);

        /// <summary>
        /// Gets the element type as an integer constant.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <returns>The element type constant (ELEMENT_TYPE_*), or -1 if invalid.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfElementGetType(IntPtr handle);

        /// <summary>
        /// Frees an element handle.
        /// </summary>
        /// <param name="handle">The element handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfElementFree(IntPtr handle);

        /// <summary>
        /// Gets the format of an image element.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The image format constant (0=JPEG, 1=PNG, 2=TIFF, 3=Unknown).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfImageElementGetFormat(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the dimensions of an image element.
        /// </summary>
        /// <param name="handle">The element handle.</param>
        /// <param name="width">Output parameter for width in pixels.</param>
        /// <param name="height">Output parameter for height in pixels.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfImageElementGetDimensions(
            IntPtr handle,
            out uint width,
            out uint height,
            out int errorCode);

        #endregion

        #region Annotation API

        /// <summary>
        /// Gets the number of annotations on a page.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The number of annotations found, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfPageGetAnnotationsCount(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the count of annotations of a specific type on a page.
        /// </summary>
        /// <param name="handle">The page handle.</param>
        /// <param name="annotationType">The type of annotation to count (ANNOTATION_TYPE_*).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The number of annotations of that type, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfPageGetAnnotationsByTypeCount(
            IntPtr handle,
            int annotationType,
            out int errorCode);

        /// <summary>
        /// Gets the type of an annotation.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <returns>The annotation type constant (ANNOTATION_TYPE_*), or -1 if invalid.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfAnnotationGetType(IntPtr handle);

        /// <summary>
        /// Gets the contents/text of an annotation.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfAnnotationGetContents(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the subject of an annotation.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfAnnotationGetSubject(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the author of an annotation.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfAnnotationGetAuthor(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the bounding box of an annotation.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="x">Output parameter for x coordinate.</param>
        /// <param name="y">Output parameter for y coordinate.</param>
        /// <param name="width">Output parameter for width.</param>
        /// <param name="height">Output parameter for height.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfAnnotationGetBbox(
            IntPtr handle,
            out float x,
            out float y,
            out float width,
            out float height);

        /// <summary>
        /// Gets the color of an annotation as RGB values (0.0-1.0).
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="r">Output parameter for red component.</param>
        /// <param name="g">Output parameter for green component.</param>
        /// <param name="b">Output parameter for blue component.</param>
        /// <param name="hasColor">Output parameter for whether color was found (1=yes, 0=no).</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfAnnotationGetColor(
            IntPtr handle,
            out float r,
            out float g,
            out float b,
            out int hasColor);

        /// <summary>
        /// Gets the opacity of an annotation (0.0-1.0).
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The opacity value (1.0 if not set).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern float PdfAnnotationGetOpacity(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets flags for an annotation (visibility, printability, etc.).
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The flags as a bitmask.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfAnnotationGetFlags(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Text annotation specific: Gets the icon type.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The icon type code (0=Comment, 1=Key, 2=Note, 3=Help, etc., -1=Unknown).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfTextAnnotationGetIcon(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Text annotation specific: Gets whether the annotation is open.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>1 if open, 0 if closed.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfTextAnnotationGetOpen(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Link annotation specific: Gets the URI of a link.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfLinkAnnotationGetUri(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Link annotation specific: Gets the destination page index.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The page index, or -1 if not a page link or error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfLinkAnnotationGetPage(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Text markup annotation specific: Gets the markup type.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The markup type (0=Highlight, 1=Underline, 2=StrikeOut, 3=Squiggly, -1=Unknown).</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfTextMarkupAnnotationGetType(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// FreeText annotation specific: Gets the font name.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfFreeTextAnnotationGetFontName(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// FreeText annotation specific: Gets the font size.
        /// </summary>
        /// <param name="handle">The annotation handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The font size in points.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern float PdfFreeTextAnnotationGetFontSize(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Frees an annotation handle.
        /// </summary>
        /// <param name="handle">The annotation handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfAnnotationFree(IntPtr handle);

        #endregion

        #region Search API

        /// <summary>
        /// Searches for text on a page.
        /// </summary>
        /// <param name="pageHandle">The page handle.</param>
        /// <param name="searchTerm">The UTF-8 search term to find.</param>
        /// <param name="caseSensitive">Whether to match case (1=yes, 0=no).</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The number of occurrences found, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention, CharSet = CharSet.Ansi)]
        public static extern int PdfPageSearchText(
            IntPtr pageHandle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string searchTerm,
            int caseSensitive,
            out int errorCode);

        /// <summary>
        /// Gets the text content of a search result.
        /// </summary>
        /// <param name="handle">The search result handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>UTF-8 null-terminated string pointer. Must be freed with FreeString().</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern IntPtr PdfSearchResultGetText(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Gets the bounding box of a search result.
        /// </summary>
        /// <param name="handle">The search result handle.</param>
        /// <param name="x">Output parameter for x coordinate.</param>
        /// <param name="y">Output parameter for y coordinate.</param>
        /// <param name="width">Output parameter for width.</param>
        /// <param name="height">Output parameter for height.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfSearchResultGetBbox(
            IntPtr handle,
            out float x,
            out float y,
            out float width,
            out float height);

        /// <summary>
        /// Gets the page index of a search result.
        /// </summary>
        /// <param name="handle">The search result handle.</param>
        /// <param name="errorCode">Output parameter for error code.</param>
        /// <returns>The page index, or -1 on error.</returns>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern int PdfSearchResultGetPage(
            IntPtr handle,
            out int errorCode);

        /// <summary>
        /// Frees a search result handle.
        /// </summary>
        /// <param name="handle">The search result handle to free.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void PdfSearchResultFree(IntPtr handle);

        #endregion

        #region Utility Functions

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
        /// <param name="len">Length of the buffer.</param>
        [DllImport(LibName, CallingConvention = DefaultCallingConvention)]
        public static extern void FreeBytes(IntPtr ptr, int len);

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
