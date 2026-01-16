package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for line annotations.
 */
public final class LineAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private LineAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static LineAnnotationBuilder create(Rect rect) {
        return new LineAnnotationBuilder(rect);
    }

    public LineAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public LineAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public LineAnnotation build() {
        return new LineAnnotation(rect, color, lineWidth);
    }
}
