package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Circle annotation.
 */
public final class CircleAnnotation extends Annotation {
    private final double[] color;
    private final double lineWidth;

    public CircleAnnotation(Rect rect, double[] color, double lineWidth) {
        super(rect);
        this.color = color;
        this.lineWidth = lineWidth;
    }

    @Override
    public String getType() {
        return "Circle";
    }

    public double[] getColor() {
        return color;
    }

    public double getLineWidth() {
        return lineWidth;
    }
}
