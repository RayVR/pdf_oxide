package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for circle annotations.
 */
public final class CircleAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private CircleAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static CircleAnnotationBuilder create(Rect rect) {
        return new CircleAnnotationBuilder(rect);
    }

    public CircleAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public CircleAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public CircleAnnotation build() {
        return new CircleAnnotation(rect, color, lineWidth);
    }
}
