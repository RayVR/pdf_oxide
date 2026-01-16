package com.pdfoxide.core;

import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.document.DocumentInfo;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.internal.NativeHandle;
import com.pdfoxide.util.NativeLibraryLoader;

import java.io.Closeable;
import java.nio.file.Path;
import java.util.Optional;

/**
 * Unified PDF API combining reading, creation, and editing capabilities.
 *
 * <p>This is the main entry point for most PDF operations. It provides:
 * <ul>
 *   <li>Factory methods for creating PDFs from various sources
 *   <li>DOM-like navigation for querying and modifying content
 *   <li>Page operations and metadata management
 *   <li>Format conversion and search
 * </ul>
 *
 * <p>Example - Reading:
 * <pre>{@code
 * try (Pdf doc = Pdf.open("input.pdf")) {
 *     int pageCount = doc.getPageCount();
 *     String text = doc.toText(0);
 *     System.out.println(text);
 * }
 * }</pre>
 *
 * <p>Example - Creating:
 * <pre>{@code
 * Pdf doc = Pdf.fromMarkdown("# Hello\n\nWorld");
 * doc.save("output.pdf");
 * }</pre>
 *
 * <p>Example - Building with configuration:
 * <pre>{@code
 * Pdf doc = PdfBuilder.create()
 *     .title("My Document")
 *     .author("John Doe")
 *     .fromMarkdown("# Content");
 * doc.save("output.pdf");
 * }</pre>
 *
 * @since 1.0.0
 */
public final class Pdf implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    /**
     * Creates a new Pdf wrapper around a native handle.
     *
     * @param handle native pointer wrapper
     */
    private Pdf(NativeHandle handle) {
        this.handle = handle;
    }

    // ===== Factory Methods =====

    /**
     * Creates a new blank PDF document.
     *
     * @return new blank PDF
     * @throws PdfException if creation fails
     */
    public static Pdf create() throws PdfException {
        long nativePtr = nativeCreate();
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Opens an existing PDF document for reading and editing.
     *
     * @param path file path
     * @return opened PDF document
     * @throws PdfException if file cannot be opened
     */
    public static Pdf open(Path path) throws PdfException {
        return open(path.toString());
    }

    /**
     * Opens an existing PDF document for reading and editing.
     *
     * @param path file path
     * @return opened PDF document
     * @throws PdfException if file cannot be opened
     */
    public static Pdf open(String path) throws PdfException {
        long nativePtr = nativeOpen(path);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Creates a PDF from Markdown source.
     *
     * @param markdown Markdown content
     * @return new PDF document
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromMarkdown(String markdown) throws PdfException {
        long nativePtr = nativeFromMarkdown(markdown);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Creates a PDF from HTML source.
     *
     * @param html HTML content
     * @return new PDF document
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromHtml(String html) throws PdfException {
        long nativePtr = nativeFromHtml(html);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Creates a PDF from plain text.
     *
     * @param text plain text content
     * @return new PDF document
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromText(String text) throws PdfException {
        long nativePtr = nativeFromText(text);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Creates a PDF from an image file.
     *
     * @param imagePath path to image (JPEG, PNG, etc.)
     * @return new PDF with image on single page
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromImage(Path imagePath) throws PdfException {
        return fromImage(imagePath.toString());
    }

    /**
     * Creates a PDF from an image file.
     *
     * @param imagePath path to image file
     * @return new PDF with image on single page
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromImage(String imagePath) throws PdfException {
        long nativePtr = nativeFromImage(imagePath);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    /**
     * Creates a PDF from multiple image files.
     *
     * @param imagePaths array of image file paths
     * @return new PDF with one image per page
     * @throws PdfException if PDF generation fails
     */
    public static Pdf fromImages(String[] imagePaths) throws PdfException {
        long nativePtr = nativeFromImages(imagePaths);
        return new Pdf(new NativeHandle(nativePtr, Pdf::nativeFree));
    }

    // ===== Page Operations =====

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
     * Saves the PDF to a file.
     *
     * @param path output file path
     * @throws PdfException if save fails
     * @throws IllegalStateException if document is closed
     */
    public void save(Path path) throws PdfException {
        save(path.toString());
    }

    /**
     * Saves the PDF to a file.
     *
     * @param path output file path
     * @throws PdfException if save fails
     * @throws IllegalStateException if document is closed
     */
    public void save(String path) throws PdfException {
        ensureNotClosed();
        nativeSave(handle.ptr(), path);
    }

    /**
     * Saves the PDF with encryption.
     *
     * @param path output file path
     * @param userPassword user password
     * @param ownerPassword owner password
     * @throws PdfException if save fails
     * @throws IllegalStateException if document is closed
     */
    public void saveEncrypted(String path, String userPassword, String ownerPassword) throws PdfException {
        ensureNotClosed();
        nativeSaveEncrypted(handle.ptr(), path, userPassword, ownerPassword);
    }

    // ===== Metadata Operations =====

    /**
     * Sets the document title.
     *
     * @param title document title
     * @throws PdfException if operation fails
     * @throws IllegalStateException if document is closed
     */
    public void setTitle(String title) throws PdfException {
        ensureNotClosed();
        nativeSetTitle(handle.ptr(), title);
    }

    /**
     * Sets the document author.
     *
     * @param author author name
     * @throws PdfException if operation fails
     * @throws IllegalStateException if document is closed
     */
    public void setAuthor(String author) throws PdfException {
        ensureNotClosed();
        nativeSetAuthor(handle.ptr(), author);
    }

    /**
     * Sets the document subject.
     *
     * @param subject subject description
     * @throws PdfException if operation fails
     * @throws IllegalStateException if document is closed
     */
    public void setSubject(String subject) throws PdfException {
        ensureNotClosed();
        nativeSetSubject(handle.ptr(), subject);
    }

    /**
     * Sets document keywords.
     *
     * @param keywords comma-separated keywords
     * @throws PdfException if operation fails
     * @throws IllegalStateException if document is closed
     */
    public void setKeywords(String keywords) throws PdfException {
        ensureNotClosed();
        nativeSetKeywords(handle.ptr(), keywords);
    }

    /**
     * Gets document information.
     *
     * @return document metadata
     * @throws PdfException if operation fails
     * @throws IllegalStateException if document is closed
     */
    public DocumentInfo getInfo() throws PdfException {
        ensureNotClosed();
        return nativeGetInfo(handle.ptr());
    }

    // ===== Conversion Operations =====

    /**
     * Converts a page to Markdown.
     *
     * @param pageIndex page index (0-based)
     * @return Markdown string
     * @throws PdfException if conversion fails
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
     * @throws IllegalStateException if document is closed
     */
    public String toMarkdown(int pageIndex, ConversionOptions options) throws PdfException {
        ensureNotClosed();
        if (options == null) {
            options = ConversionOptions.defaults();
        }
        return nativeToMarkdown(handle.ptr(), pageIndex);
    }

    /**
     * Converts a page to HTML.
     *
     * @param pageIndex page index (0-based)
     * @return HTML string
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toHtml(int pageIndex) throws PdfException {
        ensureNotClosed();
        return nativeToHtml(handle.ptr(), pageIndex);
    }

    /**
     * Converts a page to plain text.
     *
     * @param pageIndex page index (0-based)
     * @return plain text string
     * @throws PdfException if conversion fails
     * @throws IllegalStateException if document is closed
     */
    public String toText(int pageIndex) throws PdfException {
        ensureNotClosed();
        return nativeToText(handle.ptr(), pageIndex);
    }

    /**
     * Checks if the document has been modified.
     *
     * @return true if modified, false otherwise
     * @throws IllegalStateException if document is closed
     */
    public boolean isModified() {
        ensureNotClosed();
        return nativeIsModified(handle.ptr());
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
            throw new IllegalStateException("Pdf has been closed");
        }
    }

    // ============ Native Method Declarations ============

    private static native long nativeCreate() throws PdfException;

    private static native long nativeOpen(String path) throws PdfException;

    private static native long nativeFromMarkdown(String markdown) throws PdfException;

    private static native long nativeFromHtml(String html) throws PdfException;

    private static native long nativeFromText(String text) throws PdfException;

    private static native long nativeFromImage(String imagePath) throws PdfException;

    private static native long nativeFromImages(String[] imagePaths) throws PdfException;

    private static native void nativeFree(long ptr);

    private static native int nativeGetPageCount(long ptr) throws PdfException;

    private static native void nativeSave(long ptr, String path) throws PdfException;

    private static native void nativeSaveEncrypted(long ptr, String path, String userPassword, String ownerPassword) throws PdfException;

    private static native void nativeSetTitle(long ptr, String title) throws PdfException;

    private static native void nativeSetAuthor(long ptr, String author) throws PdfException;

    private static native void nativeSetSubject(long ptr, String subject) throws PdfException;

    private static native void nativeSetKeywords(long ptr, String keywords) throws PdfException;

    private static native DocumentInfo nativeGetInfo(long ptr) throws PdfException;

    private static native String nativeToMarkdown(long ptr, int pageIndex) throws PdfException;

    private static native String nativeToHtml(long ptr, int pageIndex) throws PdfException;

    private static native String nativeToText(long ptr, int pageIndex) throws PdfException;

    private static native boolean nativeIsModified(long ptr);

    static {
        try {
            NativeLibraryLoader.load();
        } catch (Exception e) {
            throw new ExceptionInInitializerError("Failed to load pdf_oxide native library: " + e.getMessage());
        }
    }
}
