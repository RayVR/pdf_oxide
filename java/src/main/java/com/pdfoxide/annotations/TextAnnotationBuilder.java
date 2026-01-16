package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for text annotations.
 */
public final class TextAnnotationBuilder {
    private final Rect rect;
    private final String content;
    private String author;
    private double[] color;

    private TextAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static TextAnnotationBuilder create(Rect rect, String content) {
        return new TextAnnotationBuilder(rect, content);
    }

    public TextAnnotationBuilder author(String author) {
        this.author = author;
        return this;
    }

    public TextAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public TextAnnotation build() {
        return new TextAnnotation(rect, content, author, color);
    }
}
