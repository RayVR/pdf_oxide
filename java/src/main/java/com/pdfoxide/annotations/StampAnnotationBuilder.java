package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for stamp annotations.
 */
public final class StampAnnotationBuilder {
    private final Rect rect;
    private final StampType stampType;
    private double[] color;

    private StampAnnotationBuilder(Rect rect, StampType stampType) {
        this.rect = rect;
        this.stampType = stampType;
    }

    public static StampAnnotationBuilder create(Rect rect, StampType stampType) {
        return new StampAnnotationBuilder(rect, stampType);
    }

    public StampAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public StampAnnotation build() {
        return new StampAnnotation(rect, stampType, color);
    }
}
