package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for watermark annotations.
 */
public final class WatermarkAnnotationBuilder {
    private final Rect rect;
    private final String text;
    private double opacity = 0.5;
    private double[] color;

    private WatermarkAnnotationBuilder(Rect rect, String text) {
        this.rect = rect;
        this.text = text;
    }

    public static WatermarkAnnotationBuilder create(Rect rect, String text) {
        return new WatermarkAnnotationBuilder(rect, text);
    }

    public WatermarkAnnotationBuilder opacity(double opacity) {
        this.opacity = opacity;
        return this;
    }

    public WatermarkAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public WatermarkAnnotation build() {
        return new WatermarkAnnotation(rect, text, opacity, color);
    }
}
