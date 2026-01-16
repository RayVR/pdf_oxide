package com.pdfoxide.creation;

import com.pdfoxide.core.Pdf;
import com.pdfoxide.exceptions.PdfException;

/**
 * Builder for creating new PDF documents.
 */
public final class DocumentBuilder {
    private PageSize pageSize = PageSize.A4;
    private double marginTop = 72.0;
    private double marginRight = 72.0;
    private double marginBottom = 72.0;
    private double marginLeft = 72.0;

    private DocumentBuilder() {
    }

    /**
     * Creates a new DocumentBuilder.
     *
     * @return new builder
     */
    public static DocumentBuilder create() {
        return new DocumentBuilder();
    }

    /**
     * Sets the page size.
     *
     * @param pageSize page size
     * @return this builder
     */
    public DocumentBuilder pageSize(PageSize pageSize) {
        this.pageSize = pageSize;
        return this;
    }

    /**
     * Sets all margins.
     *
     * @param top top margin
     * @param right right margin
     * @param bottom bottom margin
     * @param left left margin
     * @return this builder
     */
    public DocumentBuilder margins(double top, double right, double bottom, double left) {
        this.marginTop = top;
        this.marginRight = right;
        this.marginBottom = bottom;
        this.marginLeft = left;
        return this;
    }

    /**
     * Builds and returns a new blank PDF.
     *
     * @return new PDF document
     * @throws PdfException if creation fails
     */
    public Pdf build() throws PdfException {
        return Pdf.create();
    }
}
