package com.pdfoxide.conversion;

/**
 * Options for PDF to format conversion (Markdown, HTML, PlainText).
 */
public final class ConversionOptions {
    private final boolean detectHeadings;
    private final boolean preserveLayout;
    private final boolean extractTables;

    private ConversionOptions(Builder builder) {
        this.detectHeadings = builder.detectHeadings;
        this.preserveLayout = builder.preserveLayout;
        this.extractTables = builder.extractTables;
    }

    public static Builder builder() {
        return new Builder();
    }

    public boolean isDetectHeadings() { return detectHeadings; }
    public boolean isPreserveLayout() { return preserveLayout; }
    public boolean isExtractTables() { return extractTables; }

    public static class Builder {
        private boolean detectHeadings = true;
        private boolean preserveLayout = false;
        private boolean extractTables = true;

        public Builder detectHeadings(boolean detect) {
            this.detectHeadings = detect;
            return this;
        }

        public Builder preserveLayout(boolean preserve) {
            this.preserveLayout = preserve;
            return this;
        }

        public Builder extractTables(boolean extract) {
            this.extractTables = extract;
            return this;
        }

        public ConversionOptions build() {
            return new ConversionOptions(this);
        }
    }
}
