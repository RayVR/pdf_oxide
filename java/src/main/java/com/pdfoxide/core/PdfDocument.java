package com.pdfoxide.core;

import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.internal.NativeHandle;
import com.pdfoxide.util.NativeLibraryLoader;

import java.io.Closeable;
import java.nio.file.Path;

/**
 * PDF document reader for text extraction and format conversion.
 *
 * <p>Provides read-only access to PDF content with automatic reading order
 * detection and multi-format export capabilities (Markdown, HTML, Plain Text).
 *
 * <p>Features:
 * <ul>
 *   <li>Automatic reading order detection
 *   <li>Multi-column layout support
 *   <li>Font recovery (70-80% character recovery)
 *   <li>Complex script support (RTL, CJK, Devanagari, Thai)
 *   <li>PDF spec compliance (ISO 32000-1:2008)
 * </ul>
 *
 * <p>Example:
 * <pre>{@code
 * try (PdfDocument doc = PdfDocument.open("document.pdf")) {
 *     int pageCount = doc.getPageCount();
 *     String text = doc.extractText(0);
 *
 *     ConversionOptions options = ConversionOptions.builder()
 *         .detectHeadings(true)
 *         .build();
 *     String markdown = doc.toMarkdown(0, options);
 *     System.out.println(markdown);
 * }
 * }</pre>
 *
 * @since 1.0.0
 */
public final class PdfDocument implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    /**
     * Creates a new PdfDocument wrapper around a native handle.
     *
     * @param handle native pointer wrapper
     */
    private PdfDocument(NativeHandle handle) {
        this.handle = handle;
    }

    /**
     * Opens a PDF document from a file path.
     *
     * @param path the file path to the PDF
     * @return opened PDF document
     * @throws PdfException if the file cannot be opened or is not a valid PDF
     */
    public static PdfDocument open(Path path) throws PdfException {
        return open(path.toString());
    }

    /**
     * Opens a PDF document from a file path string.
     *
     * @param path the file path to the PDF
     * @return opened PDF document
     * @throws PdfException if the file cannot be opened or is not a valid PDF
     */
    public static PdfDocument open(String path) throws PdfException {
        long nativePtr = nativeOpen(path);
        return new PdfDocument(new NativeHandle(nativePtr, PdfDocument::nativeFree));
    }

    /**
     * Opens an encrypted PDF with a password.
     *
     * @param path the file path
     * @param password user or owner password
     * @return opened PDF document
     * @throws PdfException if password is incorrect or file cannot be opened
     */
    public static PdfDocument openWithPassword(String path, String password) throws PdfException {
        long nativePtr = nativeOpenWithPassword(path, password);
        return new PdfDocument(new NativeHandle(nativePtr, PdfDocument::nativeFree));
    }

    /**
     * Opens an encrypted PDF with a password from a file path.
     *
     * @param path the file path
     * @param password user or owner password
     * @return opened PDF document
     * @throws PdfException if password is incorrect or file cannot be opened
     */
    public static PdfDocument openWithPassword(Path path, String password) throws PdfException {
        return openWithPassword(path.toString(), password);
    }

    /**
     * Gets the PDF version.
     *
     * @return version as [major, minor], e.g., [1, 7] for PDF 1.7
     * @throws PdfException if version cannot be determined
     * @throws IllegalStateException if document is closed
     */
    public int[] getVersion() throws PdfException {
        ensureNotClosed();
        return nativeGetVersion(handle.ptr());
    }

    /**
     * Gets the number of pages in the document.
     *
     * @return page count
     * @throws PdfException if page count cannot be determined
     * @throws IllegalStateException if document is closed
     */
    public int getPageCount() throws PdfException {
        ensureNotClosed();
        return nativeGetPageCount(handle.ptr());
    }

    /**
     * Extracts text from the specified page with automatic reading order.
     *
     * @param pageIndex page index (0-based)
     * @return extracted text
     * @throws PdfException if extraction fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String extractText(int pageIndex) throws PdfException {
        ensureNotClosed();
        return nativeExtractText(handle.ptr(), pageIndex);
    }

    /**
     * Converts a page to Markdown with default options.
     *
     * @param pageIndex page index (0-based)
     * @return Markdown string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toMarkdown(int pageIndex) throws PdfException {
        return toMarkdown(pageIndex, ConversionOptions.defaults());
    }

    /**
     * Converts a page to Markdown with custom options.
     *
     * @param pageIndex page index (0-based)
     * @param options conversion configuration
     * @return Markdown string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toMarkdown(int pageIndex, ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        long optionsPtr = nativeCreateConversionOptions(
            options.isDetectHeadings(),
            options.isPreserveLayout(),
            options.isExtractImages(),
            options.isExtractTables(),
            options.getMaxLineLength(),
            options.getLanguageHints()
        );
        try {
            return nativeToMarkdown(handle.ptr(), pageIndex, optionsPtr);
        } finally {
            nativeFreeConversionOptions(optionsPtr);
        }
    }

    /**
     * Converts all pages to Markdown with default options.
     *
     * @return Markdown string for entire document
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toMarkdownAll() throws PdfException {
        return toMarkdownAll(ConversionOptions.defaults());
    }

    /**
     * Converts all pages to Markdown with custom options.
     *
     * @param options conversion configuration
     * @return Markdown string for entire document
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toMarkdownAll(ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        long optionsPtr = nativeCreateConversionOptions(
            options.isDetectHeadings(),
            options.isPreserveLayout(),
            options.isExtractImages(),
            options.isExtractTables(),
            options.getMaxLineLength(),
            options.getLanguageHints()
        );
        try {
            return nativeToMarkdownAll(handle.ptr(), optionsPtr);
        } finally {
            nativeFreeConversionOptions(optionsPtr);
        }
    }

    /**
     * Converts a page to HTML with default options.
     *
     * @param pageIndex page index (0-based)
     * @return HTML string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toHtml(int pageIndex) throws PdfException {
        return toHtml(pageIndex, ConversionOptions.defaults());
    }

    /**
     * Converts a page to HTML with custom options.
     *
     * @param pageIndex page index (0-based)
     * @param options conversion configuration
     * @return HTML string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toHtml(int pageIndex, ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        long optionsPtr = nativeCreateConversionOptions(
            options.isDetectHeadings(),
            options.isPreserveLayout(),
            options.isExtractImages(),
            options.isExtractTables(),
            options.getMaxLineLength(),
            options.getLanguageHints()
        );
        try {
            return nativeToHtml(handle.ptr(), pageIndex, optionsPtr);
        } finally {
            nativeFreeConversionOptions(optionsPtr);
        }
    }

    /**
     * Converts all pages to HTML with default options.
     *
     * @return HTML string for entire document
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toHtmlAll() throws PdfException {
        return toHtmlAll(ConversionOptions.defaults());
    }

    /**
     * Converts all pages to HTML with custom options.
     *
     * @param options conversion configuration
     * @return HTML string for entire document
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toHtmlAll(ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        long optionsPtr = nativeCreateConversionOptions(
            options.isDetectHeadings(),
            options.isPreserveLayout(),
            options.isExtractImages(),
            options.isExtractTables(),
            options.getMaxLineLength(),
            options.getLanguageHints()
        );
        try {
            return nativeToHtmlAll(handle.ptr(), optionsPtr);
        } finally {
            nativeFreeConversionOptions(optionsPtr);
        }
    }

    /**
     * Converts a page to plain text with default options.
     *
     * @param pageIndex page index (0-based)
     * @return plain text string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toPlainText(int pageIndex) throws PdfException {
        return toPlainText(pageIndex, ConversionOptions.defaults());
    }

    /**
     * Converts a page to plain text with custom options.
     *
     * @param pageIndex page index (0-based)
     * @param options conversion configuration
     * @return plain text string
     * @throws PdfException if conversion fails
     * @throws IllegalArgumentException if page index is out of bounds
     * @throws IllegalStateException if document is closed
     */
    public String toPlainText(int pageIndex, ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        long optionsPtr = nativeCreateConversionOptions(
            options.isDetectHeadings(),
            options.isPreserveLayout(),
            options.isExtractImages(),
            options.isExtractTables(),
            options.getMaxLineLength(),
            options.getLanguageHints()
        );
        try {
            return nativeToPlainText(handle.ptr(), pageIndex, optionsPtr);
        } finally {
            nativeFreeConversionOptions(optionsPtr);
        }
    }

    /**
     * Checks if the document has a logical structure tree (Tagged PDF).
     *
     * @return true if Tagged PDF with structure tree, false otherwise
     * @throws IllegalStateException if document is closed
     */
    public boolean hasStructureTree() {
        ensureNotClosed();
        return nativeHasStructureTree(handle.ptr());
    }

    @Override
    public void close() {
        if (!closed) {
            handle.close();
            closed = true;
        }
    }

    /**
     * Checks if the document is closed.
     *
     * @return true if closed
     */
    public boolean isClosed() {
        return closed;
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("PdfDocument has been closed");
        }
    }

    // ============ Native Method Declarations ============

    private static native long nativeOpen(String path) throws PdfException;

    private static native long nativeOpenWithPassword(String path, String password) throws PdfException;

    private static native void nativeFree(long ptr);

    private static native int[] nativeGetVersion(long ptr) throws PdfException;

    private static native int nativeGetPageCount(long ptr) throws PdfException;

    private static native String nativeExtractText(long ptr, int pageIndex) throws PdfException;

    private static native long nativeCreateConversionOptions(
        boolean detectHeadings,
        boolean preserveLayout,
        boolean extractImages,
        boolean extractTables,
        int maxLineLength,
        String languageHints
    );

    private static native void nativeFreeConversionOptions(long optionsPtr);

    private static native String nativeToMarkdown(long ptr, int pageIndex, long optionsPtr) throws PdfException;

    private static native String nativeToMarkdownAll(long ptr, long optionsPtr) throws PdfException;

    private static native String nativeToHtml(long ptr, int pageIndex, long optionsPtr) throws PdfException;

    private static native String nativeToHtmlAll(long ptr, long optionsPtr) throws PdfException;

    private static native String nativeToPlainText(long ptr, int pageIndex, long optionsPtr) throws PdfException;

    private static native boolean nativeHasStructureTree(long ptr);

    static {
        try {
            NativeLibraryLoader.load();
        } catch (Exception e) {
            throw new ExceptionInInitializerError("Failed to load pdf_oxide native library: " + e.getMessage());
        }
    }
}
