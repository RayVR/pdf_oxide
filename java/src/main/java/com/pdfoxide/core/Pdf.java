package com.pdfoxide.core;

import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.dom.PdfPage;
import com.pdfoxide.exceptions.PdfException;
import java.nio.file.Path;

/**
 * Unified PDF API combining reading, creation, and editing capabilities.
 * Main entry point for most PDF operations.
 */
public final class Pdf implements AutoCloseable {
    private long nativePtr;
    private boolean closed = false;

    private Pdf(long nativePtr) {
        this.nativePtr = nativePtr;
    }

    public static Pdf open(Path path) throws PdfException {
        return open(path.toString());
    }

    public static Pdf open(String path) throws PdfException {
        long ptr = nativeOpen(path);
        return new Pdf(ptr);
    }

    public static Pdf fromMarkdown(String markdown) throws PdfException {
        long ptr = nativeFromMarkdown(markdown);
        return new Pdf(ptr);
    }

    public static Pdf fromHtml(String html) throws PdfException {
        long ptr = nativeFromHtml(html);
        return new Pdf(ptr);
    }

    public static Pdf fromText(String text) throws PdfException {
        long ptr = nativeFromText(text);
        return new Pdf(ptr);
    }

    public static Pdf fromImage(Path imagePath) throws PdfException {
        return fromImage(imagePath.toString());
    }

    public static Pdf fromImage(String imagePath) throws PdfException {
        long ptr = nativeFromImage(imagePath);
        return new Pdf(ptr);
    }

    public PdfPage getPage(int index) throws PdfException {
        if (closed) throw new IllegalStateException("Pdf closed");
        long pagePtr = nativeGetPage(nativePtr, index);
        return new PdfPage(pagePtr, index);
    }

    public int getPageCount() throws PdfException {
        if (closed) throw new IllegalStateException("Pdf closed");
        return nativeGetPageCount(nativePtr);
    }

    public void savePage(PdfPage page) throws PdfException {
        if (closed) throw new IllegalStateException("Pdf closed");
        nativeSavePage(nativePtr, page.getNativePtr());
    }

    public void save(Path path) throws PdfException {
        save(path.toString());
    }

    public void save(String path) throws PdfException {
        if (closed) throw new IllegalStateException("Pdf closed");
        nativeSave(nativePtr, path);
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
    private static native long nativeFromMarkdown(String markdown) throws PdfException;
    private static native long nativeFromHtml(String html) throws PdfException;
    private static native long nativeFromText(String text) throws PdfException;
    private static native long nativeFromImage(String imagePath) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native long nativeGetPage(long ptr, int index) throws PdfException;
    private static native int nativeGetPageCount(long ptr) throws PdfException;
    private static native void nativeSavePage(long pdfPtr, long pagePtr) throws PdfException;
    private static native void nativeSave(long ptr, String path) throws PdfException;
}
