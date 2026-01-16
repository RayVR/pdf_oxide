package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for free text annotations.
 */
public final class FreeTextAnnotationBuilder {
    private final Rect rect;
    private final String content;
    private String fontName = "Helvetica";
    private double fontSize = 12.0;
    private double[] color;

    private FreeTextAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static FreeTextAnnotationBuilder create(Rect rect, String content) {
        return new FreeTextAnnotationBuilder(rect, content);
    }

    public FreeTextAnnotationBuilder fontName(String fontName) {
        this.fontName = fontName;
        return this;
    }

    public FreeTextAnnotationBuilder fontSize(double fontSize) {
        this.fontSize = fontSize;
        return this;
    }

    public FreeTextAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public FreeTextAnnotation build() {
        return new FreeTextAnnotation(rect, content, fontName, fontSize, color);
    }
}
