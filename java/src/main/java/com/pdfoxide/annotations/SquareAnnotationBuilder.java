package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for square annotations.
 */
public final class SquareAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private SquareAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static SquareAnnotationBuilder create(Rect rect) {
        return new SquareAnnotationBuilder(rect);
    }

    public SquareAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public SquareAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public SquareAnnotation build() {
        return new SquareAnnotation(rect, color, lineWidth);
    }
}
