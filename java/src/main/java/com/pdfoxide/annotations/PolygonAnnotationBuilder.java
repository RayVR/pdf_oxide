package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for polygon annotations.
 */
public final class PolygonAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private PolygonAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static PolygonAnnotationBuilder create(Rect rect) {
        return new PolygonAnnotationBuilder(rect);
    }

    public PolygonAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public PolygonAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public PolygonAnnotation build() {
        return new PolygonAnnotation(rect, color, lineWidth);
    }
}
