package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Square annotation.
 */
public final class SquareAnnotation extends Annotation {
    private final double[] color;
    private final double lineWidth;

    public SquareAnnotation(Rect rect, double[] color, double lineWidth) {
        super(rect);
        this.color = color;
        this.lineWidth = lineWidth;
    }

    @Override
    public String getType() {
        return "Square";
    }

    public double[] getColor() {
        return color;
    }

    public double getLineWidth() {
        return lineWidth;
    }
}
