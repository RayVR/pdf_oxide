package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Squiggly annotation.
 */
public final class SquigglyAnnotation extends Annotation {
    private final double[] color;

    public SquigglyAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "Squiggly";
    }

    public double[] getColor() {
        return color;
    }
}
