package com.pdfoxide.core;

import com.pdfoxide.creation.PageSize;
import com.pdfoxide.exceptions.PdfException;

/**
 * Builder for creating PDF documents with custom configuration.
 *
 * <p>Provides fluent API for configuring document metadata, page properties,
 * and creation options before generating the PDF.
 *
 * <p>Example:
 * <pre>{@code
 * Pdf doc = PdfBuilder.create()
 *     .title("My Document")
 *     .author("John Doe")
 *     .subject("Example")
 *     .pageSize(PageSize.A4)
 *     .margins(72.0, 72.0, 72.0, 72.0)
 *     .fromMarkdown("# Hello\n\nWorld");
 * doc.save("output.pdf");
 * }</pre>
 */
public final class PdfBuilder {
    private String title;
    private String author;
    private String subject;
    private String keywords;
    private PageSize pageSize = PageSize.A4;
    private double marginTop = 72.0;
    private double marginRight = 72.0;
    private double marginBottom = 72.0;
    private double marginLeft = 72.0;
    private boolean compressContent = true;
    private boolean embedFonts = true;

    private PdfBuilder() {
    }

    /**
     * Creates a new PdfBuilder instance.
     *
     * @return new builder
     */
    public static PdfBuilder create() {
        return new PdfBuilder();
    }

    /**
     * Sets the document title.
     *
     * @param title document title
     * @return this builder
     */
    public PdfBuilder title(String title) {
        this.title = title;
        return this;
    }

    /**
     * Sets the document author.
     *
     * @param author author name
     * @return this builder
     */
    public PdfBuilder author(String author) {
        this.author = author;
        return this;
    }

    /**
     * Sets the document subject.
     *
     * @param subject subject description
     * @return this builder
     */
    public PdfBuilder subject(String subject) {
        this.subject = subject;
        return this;
    }

    /**
     * Sets document keywords.
     *
     * @param keywords comma-separated keywords
     * @return this builder
     */
    public PdfBuilder keywords(String keywords) {
        this.keywords = keywords;
        return this;
    }

    /**
     * Sets the page size.
     *
     * @param pageSize page size enum
     * @return this builder
     */
    public PdfBuilder pageSize(PageSize pageSize) {
        this.pageSize = pageSize;
        return this;
    }

    /**
     * Sets all margins at once.
     *
     * @param top top margin in points
     * @param right right margin in points
     * @param bottom bottom margin in points
     * @param left left margin in points
     * @return this builder
     */
    public PdfBuilder margins(double top, double right, double bottom, double left) {
        this.marginTop = top;
        this.marginRight = right;
        this.marginBottom = bottom;
        this.marginLeft = left;
        return this;
    }

    /**
     * Sets whether to compress content streams.
     *
     * @param compress compress flag
     * @return this builder
     */
    public PdfBuilder compressContent(boolean compress) {
        this.compressContent = compress;
        return this;
    }

    /**
     * Sets whether to embed fonts.
     *
     * @param embed embed flag
     * @return this builder
     */
    public PdfBuilder embedFonts(boolean embed) {
        this.embedFonts = embed;
        return this;
    }

    /**
     * Creates a PDF from Markdown.
     *
     * @param markdown Markdown content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromMarkdown(String markdown) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }

    /**
     * Creates a PDF from HTML.
     *
     * @param html HTML content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromHtml(String html) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }

    /**
     * Creates a PDF from plain text.
     *
     * @param text plain text content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromText(String text) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }
}
