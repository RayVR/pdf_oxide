package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Underline annotation.
 */
public final class UnderlineAnnotation extends Annotation {
    private final double[] color;

    public UnderlineAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "Underline";
    }

    public double[] getColor() {
        return color;
    }
}
