package com.pdfoxide.core;

import com.pdfoxide.creation.PageSize;
import com.pdfoxide.exceptions.PdfException;

/**
 * Fluent builder for creating PDFs with customizable metadata and layout.
 */
public final class PdfBuilder {
    private String title;
    private String author;
    private String subject;
    private String[] keywords;
    private PageSize pageSize = PageSize.LETTER;
    private double marginTop = 72.0;
    private double marginRight = 72.0;
    private double marginBottom = 72.0;
    private double marginLeft = 72.0;

    private PdfBuilder() {}

    public static PdfBuilder create() {
        return new PdfBuilder();
    }

    public PdfBuilder title(String title) {
        this.title = title;
        return this;
    }

    public PdfBuilder author(String author) {
        this.author = author;
        return this;
    }

    public PdfBuilder subject(String subject) {
        this.subject = subject;
        return this;
    }

    public PdfBuilder keywords(String... keywords) {
        this.keywords = keywords;
        return this;
    }

    public PdfBuilder pageSize(PageSize pageSize) {
        this.pageSize = pageSize;
        return this;
    }

    public PdfBuilder margins(double top, double right, double bottom, double left) {
        this.marginTop = top;
        this.marginRight = right;
        this.marginBottom = bottom;
        this.marginLeft = left;
        return this;
    }

    public Pdf fromMarkdown(String markdown) throws PdfException {
        return buildFrom(markdown, ContentType.MARKDOWN);
    }

    public Pdf fromHtml(String html) throws PdfException {
        return buildFrom(html, ContentType.HTML);
    }

    public Pdf fromText(String text) throws PdfException {
        return buildFrom(text, ContentType.TEXT);
    }

    private Pdf buildFrom(String content, ContentType type) throws PdfException {
        long configPtr = nativeCreateConfig(
            title, author, subject, keywords,
            pageSize.ordinal(),
            marginTop, marginRight, marginBottom, marginLeft
        );

        long pdfPtr = nativeBuildFrom(configPtr, content, type.ordinal());
        nativeFreeConfig(configPtr);

        return new Pdf(pdfPtr);
    }

    private enum ContentType {
        MARKDOWN, HTML, TEXT
    }

    private static native long nativeCreateConfig(
        String title, String author, String subject, String[] keywords,
        int pageSizeOrdinal,
        double marginTop, double marginRight, double marginBottom, double marginLeft
    );
    private static native long nativeBuildFrom(long configPtr, String content, int typeOrdinal) throws PdfException;
    private static native void nativeFreeConfig(long ptr);
}
