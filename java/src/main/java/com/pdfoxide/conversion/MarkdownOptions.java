package com.pdfoxide.conversion;

/**
 * Options specific to Markdown conversion.
 */
public final class MarkdownOptions {
    private final boolean detectHeadings;
    private final boolean preserveLayout;
    private final boolean createTableOfContents;

    public MarkdownOptions(boolean detectHeadings, boolean preserveLayout, boolean createTableOfContents) {
        this.detectHeadings = detectHeadings;
        this.preserveLayout = preserveLayout;
        this.createTableOfContents = createTableOfContents;
    }

    public boolean isDetectHeadings() {
        return detectHeadings;
    }

    public boolean isPreserveLayout() {
        return preserveLayout;
    }

    public boolean isCreateTableOfContents() {
        return createTableOfContents;
    }
}
