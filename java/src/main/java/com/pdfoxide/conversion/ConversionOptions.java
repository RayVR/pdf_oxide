package com.pdfoxide.conversion;

/**
 * Configuration options for PDF format conversion (Markdown, HTML, Plain Text).
 *
 * <p>Use {@link #builder()} to create instances with customized settings.
 *
 * <p>Example:
 * <pre>{@code
 * ConversionOptions options = ConversionOptions.builder()
 *     .detectHeadings(true)
 *     .preserveLayout(false)
 *     .build();
 *
 * String markdown = doc.toMarkdown(0, options);
 * }</pre>
 *
 * @since 1.0.0
 */
public final class ConversionOptions {
    private final boolean detectHeadings;
    private final boolean preserveLayout;
    private final boolean extractImages;
    private final boolean extractTables;
    private final int maxLineLength;
    private final String languageHints;

    private ConversionOptions(Builder builder) {
        this.detectHeadings = builder.detectHeadings;
        this.preserveLayout = builder.preserveLayout;
        this.extractImages = builder.extractImages;
        this.extractTables = builder.extractTables;
        this.maxLineLength = builder.maxLineLength;
        this.languageHints = builder.languageHints;
    }

    /**
     * Creates a new builder for ConversionOptions.
     *
     * @return builder with default settings
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * Creates ConversionOptions with default settings.
     *
     * @return default options
     */
    public static ConversionOptions defaults() {
        return builder().build();
    }

    // Getters

    /**
     * Checks if heading detection is enabled.
     *
     * @return true if headings should be detected (default: false)
     */
    public boolean isDetectHeadings() {
        return detectHeadings;
    }

    /**
     * Checks if layout preservation is enabled.
     *
     * @return true if layout should be preserved (default: false)
     */
    public boolean isPreserveLayout() {
        return preserveLayout;
    }

    /**
     * Checks if image extraction is enabled.
     *
     * @return true if images should be extracted (default: false)
     */
    public boolean isExtractImages() {
        return extractImages;
    }

    /**
     * Checks if table extraction is enabled.
     *
     * @return true if tables should be extracted (default: true)
     */
    public boolean isExtractTables() {
        return extractTables;
    }

    /**
     * Gets the maximum line length in characters.
     *
     * @return max line length (default: 80)
     */
    public int getMaxLineLength() {
        return maxLineLength;
    }

    /**
     * Gets language hints for text extraction.
     *
     * @return language hint string or empty string (default: "")
     */
    public String getLanguageHints() {
        return languageHints;
    }

    /**
     * Builder for ConversionOptions.
     */
    public static final class Builder {
        private boolean detectHeadings = false;
        private boolean preserveLayout = false;
        private boolean extractImages = false;
        private boolean extractTables = true;
        private int maxLineLength = 80;
        private String languageHints = "";

        /**
         * Sets whether to detect and format headings.
         *
         * @param detectHeadings true to detect headings
         * @return this builder
         */
        public Builder detectHeadings(boolean detectHeadings) {
            this.detectHeadings = detectHeadings;
            return this;
        }

        /**
         * Sets whether to preserve page layout (margins, spacing, etc.).
         *
         * @param preserveLayout true to preserve layout
         * @return this builder
         */
        public Builder preserveLayout(boolean preserveLayout) {
            this.preserveLayout = preserveLayout;
            return this;
        }

        /**
         * Sets whether to extract images as embedded or referenced files.
         *
         * @param extractImages true to extract images
         * @return this builder
         */
        public Builder extractImages(boolean extractImages) {
            this.extractImages = extractImages;
            return this;
        }

        /**
         * Sets whether to extract and format tables.
         *
         * @param extractTables true to extract tables
         * @return this builder
         */
        public Builder extractTables(boolean extractTables) {
            this.extractTables = extractTables;
            return this;
        }

        /**
         * Sets the maximum line length for text wrapping.
         *
         * @param maxLineLength max characters per line (default: 80)
         * @return this builder
         */
        public Builder maxLineLength(int maxLineLength) {
            if (maxLineLength < 20) {
                throw new IllegalArgumentException("maxLineLength must be at least 20");
            }
            this.maxLineLength = maxLineLength;
            return this;
        }

        /**
         * Sets language hints for improved text extraction.
         *
         * @param languageHints language codes (e.g., "en,ja" for English and Japanese)
         * @return this builder
         */
        public Builder languageHints(String languageHints) {
            this.languageHints = languageHints != null ? languageHints : "";
            return this;
        }

        /**
         * Builds the ConversionOptions instance.
         *
         * @return immutable ConversionOptions
         */
        public ConversionOptions build() {
            return new ConversionOptions(this);
        }
    }

    @Override
    public String toString() {
        return "ConversionOptions{" +
                "detectHeadings=" + detectHeadings +
                ", preserveLayout=" + preserveLayout +
                ", extractImages=" + extractImages +
                ", extractTables=" + extractTables +
                ", maxLineLength=" + maxLineLength +
                ", languageHints='" + languageHints + '\'' +
                '}';
    }
}
