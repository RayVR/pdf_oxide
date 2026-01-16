package com.pdfoxide.core;

import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.exceptions.PdfException;
import java.nio.file.Path;

/**
 * PDF document reader for text extraction and format conversion.
 * Provides read-only access to PDF content.
 */
public final class PdfDocument implements AutoCloseable {
    private long nativePtr;
    private boolean closed = false;

    private PdfDocument(long nativePtr) {
        this.nativePtr = nativePtr;
    }

    public static PdfDocument open(Path path) throws PdfException {
        return open(path.toString());
    }

    public static PdfDocument open(String path) throws PdfException {
        long ptr = nativeOpen(path);
        return new PdfDocument(ptr);
    }

    public int[] getVersion() {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeGetVersion(nativePtr);
    }

    public int getPageCount() throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeGetPageCount(nativePtr);
    }

    public String extractText(int page) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeExtractText(nativePtr, page);
    }

    public String toMarkdown(int page) throws PdfException {
        return toMarkdown(page, ConversionOptions.builder().build());
    }

    public String toMarkdown(int page, ConversionOptions options) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeToMarkdown(nativePtr, page, options);
    }

    public String toMarkdownAll(ConversionOptions options) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeToMarkdownAll(nativePtr, options);
    }

    public String toHtml(int page, ConversionOptions options) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeToHtml(nativePtr, page, options);
    }

    public String toHtmlAll(ConversionOptions options) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeToHtmlAll(nativePtr, options);
    }

    public String toPlainText(int page, ConversionOptions options) throws PdfException {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeToPlainText(nativePtr, page, options);
    }

    public boolean hasStructureTree() {
        if (closed) throw new IllegalStateException("Document closed");
        return nativeHasStructureTree(nativePtr);
    }

    @Override
    public void close() {
        if (!closed) {
            nativeFree(nativePtr);
            closed = true;
        }
    }

    // Native methods
    private static native long nativeOpen(String path) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native int[] nativeGetVersion(long ptr);
    private static native int nativeGetPageCount(long ptr) throws PdfException;
    private static native String nativeExtractText(long ptr, int page) throws PdfException;
    private static native String nativeToMarkdown(long ptr, int page, ConversionOptions options) throws PdfException;
    private static native String nativeToMarkdownAll(long ptr, ConversionOptions options) throws PdfException;
    private static native String nativeToHtml(long ptr, int page, ConversionOptions options) throws PdfException;
    private static native String nativeToHtmlAll(long ptr, ConversionOptions options) throws PdfException;
    private static native String nativeToPlainText(long ptr, int page, ConversionOptions options) throws PdfException;
    private static native boolean nativeHasStructureTree(long ptr);
}
