package com.pdfoxide.document;

import com.pdfoxide.annotations.Annotation;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.forms.FormField;
import com.pdfoxide.forms.FormFieldValue;
import com.pdfoxide.internal.NativeHandle;

import java.io.Closeable;
import java.nio.file.Path;

/**
 * API for editing PDF documents, adding annotations and form fields.
 */
public final class DocumentEditor implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    private DocumentEditor(NativeHandle handle) {
        this.handle = handle;
    }

    /**
     * Opens a PDF document for editing.
     *
     * @param path file path
     * @return document editor
     * @throws PdfException if file cannot be opened
     */
    public static DocumentEditor open(String path) throws PdfException {
        long nativePtr = nativeOpen(path);
        return new DocumentEditor(new NativeHandle(nativePtr, DocumentEditor::nativeFree));
    }

    /**
     * Opens a PDF document for editing.
     *
     * @param path file path
     * @return document editor
     * @throws PdfException if file cannot be opened
     */
    public static DocumentEditor open(Path path) throws PdfException {
        return open(path.toString());
    }

    /**
     * Adds an annotation to a page.
     *
     * @param pageIndex page index (0-based)
     * @param annotation annotation to add
     * @throws PdfException if operation fails
     */
    public void addAnnotation(int pageIndex, Annotation annotation) throws PdfException {
        ensureNotClosed();
        nativeAddAnnotation(handle.ptr(), pageIndex, annotation);
    }

    /**
     * Adds a form field to a page.
     *
     * @param pageIndex page index (0-based)
     * @param field form field to add
     * @throws PdfException if operation fails
     */
    public void addFormField(int pageIndex, FormField field) throws PdfException {
        ensureNotClosed();
        nativeAddFormField(handle.ptr(), pageIndex, field);
    }

    /**
     * Sets a form field value.
     *
     * @param fieldName field name
     * @param value field value
     * @throws PdfException if operation fails
     */
    public void setFormFieldValue(String fieldName, FormFieldValue value) throws PdfException {
        ensureNotClosed();
        nativeSetFormFieldValue(handle.ptr(), fieldName, value);
    }

    /**
     * Saves the edited PDF.
     *
     * @param path output file path
     * @throws PdfException if save fails
     */
    public void save(String path) throws PdfException {
        ensureNotClosed();
        nativeSave(handle.ptr(), path);
    }

    /**
     * Saves the edited PDF.
     *
     * @param path output file path
     * @throws PdfException if save fails
     */
    public void save(Path path) throws PdfException {
        save(path.toString());
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
            throw new IllegalStateException("DocumentEditor has been closed");
        }
    }

    private static native long nativeOpen(String path) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native void nativeAddAnnotation(long ptr, int pageIndex, Annotation annotation) throws PdfException;
    private static native void nativeAddFormField(long ptr, int pageIndex, FormField field) throws PdfException;
    private static native void nativeSetFormFieldValue(long ptr, String fieldName, FormFieldValue value) throws PdfException;
    private static native void nativeSave(long ptr, String path) throws PdfException;
}
