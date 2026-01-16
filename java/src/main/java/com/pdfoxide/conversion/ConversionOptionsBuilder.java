package com.pdfoxide.conversion;

/**
 * Builder for ConversionOptions.
 */
public final class ConversionOptionsBuilder {
    private boolean detectHeadings = true;
    private boolean preserveLayout = true;
    private boolean includeImages = true;
    private int jpegQuality = 85;
    private String outputEncoding = "UTF-8";

    public ConversionOptionsBuilder detectHeadings(boolean detect) {
        this.detectHeadings = detect;
        return this;
    }

    public ConversionOptionsBuilder preserveLayout(boolean preserve) {
        this.preserveLayout = preserve;
        return this;
    }

    public ConversionOptionsBuilder includeImages(boolean include) {
        this.includeImages = include;
        return this;
    }

    public ConversionOptionsBuilder jpegQuality(int quality) {
        this.jpegQuality = quality;
        return this;
    }

    public ConversionOptionsBuilder outputEncoding(String encoding) {
        this.outputEncoding = encoding;
        return this;
    }

    public ConversionOptions build() {
        return new ConversionOptions(detectHeadings, preserveLayout, includeImages, jpegQuality, outputEncoding);
    }
}
