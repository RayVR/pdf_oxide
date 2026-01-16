package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for caret annotations.
 */
public final class CaretAnnotationBuilder {
    private final Rect rect;
    private Caret caretType = Caret.P;
    private double[] color;

    private CaretAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static CaretAnnotationBuilder create(Rect rect) {
        return new CaretAnnotationBuilder(rect);
    }

    public CaretAnnotationBuilder caretType(Caret type) {
        this.caretType = type;
        return this;
    }

    public CaretAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public CaretAnnotation build() {
        return new CaretAnnotation(rect, caretType, color);
    }
}
