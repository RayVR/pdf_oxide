package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for redact annotations.
 */
public final class RedactAnnotationBuilder {
    private final Rect rect;
    private double[] color = new double[]{0.0, 0.0, 0.0};
    private String replacementText;

    private RedactAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static RedactAnnotationBuilder create(Rect rect) {
        return new RedactAnnotationBuilder(rect);
    }

    public RedactAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public RedactAnnotationBuilder replacementText(String text) {
        this.replacementText = text;
        return this;
    }

    public RedactAnnotation build() {
        return new RedactAnnotation(rect, color, replacementText);
    }
}
