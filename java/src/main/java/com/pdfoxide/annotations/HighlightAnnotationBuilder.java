package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for highlight annotations.
 */
public final class HighlightAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double opacity = 1.0;

    private HighlightAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static HighlightAnnotationBuilder create(Rect rect) {
        return new HighlightAnnotationBuilder(rect);
    }

    public HighlightAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public HighlightAnnotationBuilder opacity(double opacity) {
        this.opacity = opacity;
        return this;
    }

    public HighlightAnnotation build() {
        return new HighlightAnnotation(rect, color, opacity);
    }
}
