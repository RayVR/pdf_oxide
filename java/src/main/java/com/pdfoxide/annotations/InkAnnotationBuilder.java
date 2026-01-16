package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for ink annotations.
 */
public final class InkAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private InkAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static InkAnnotationBuilder create(Rect rect) {
        return new InkAnnotationBuilder(rect);
    }

    public InkAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public InkAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public InkAnnotation build() {
        return new InkAnnotation(rect, color, lineWidth);
    }
}
