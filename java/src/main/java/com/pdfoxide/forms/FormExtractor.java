package com.pdfoxide.forms;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;
import java.util.List;

/**
 * Extracts form fields from PDF documents.
 */
public final class FormExtractor {
    private final PdfDocument document;

    public FormExtractor(PdfDocument document) {
        this.document = document;
    }

    /**
     * Extracts all form fields.
     *
     * @return list of form fields
     * @throws PdfException if extraction fails
     */
    public List<FormField> extractFields() throws PdfException {
        return nativeExtractFields(document);
    }

    /**
     * Exports form data to FDF format.
     *
     * @param path output file path
     * @throws PdfException if export fails
     */
    public void exportFdf(String path) throws PdfException {
        nativeExportFdf(document, path);
    }

    /**
     * Exports form data to XFDF format.
     *
     * @param path output file path
     * @throws PdfException if export fails
     */
    public void exportXfdf(String path) throws PdfException {
        nativeExportXfdf(document, path);
    }

    public void close() throws PdfException {
        // Cleanup if needed
    }

    private static native List<FormField> nativeExtractFields(PdfDocument document) throws PdfException;
    private static native void nativeExportFdf(PdfDocument document, String path) throws PdfException;
    private static native void nativeExportXfdf(PdfDocument document, String path) throws PdfException;
}
