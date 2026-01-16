package com.pdfoxide.search;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.internal.NativeHandle;

import java.io.Closeable;
import java.util.List;

/**
 * Text search functionality for PDF documents.
 */
public final class TextSearcher implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    public TextSearcher(PdfDocument document) throws PdfException {
        long nativePtr = nativeCreate(document);
        this.handle = new NativeHandle(nativePtr, TextSearcher::nativeFree);
    }

    /**
     * Searches for text in the document.
     *
     * @param query search query
     * @param options search options
     * @return list of search results
     * @throws PdfException if search fails
     */
    public List<SearchResult> search(String query, SearchOptions options) throws PdfException {
        ensureNotClosed();
        return nativeSearch(handle.ptr(), query, options);
    }

    @Override
    public void close() {
        if (!closed) {
            handle.close();
            closed = true;
        }
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("TextSearcher has been closed");
        }
    }

    private static native long nativeCreate(PdfDocument document) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native List<SearchResult> nativeSearch(long ptr, String query, SearchOptions options) throws PdfException;
}
