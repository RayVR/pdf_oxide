package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Strike-out annotation.
 */
public final class StrikeOutAnnotation extends Annotation {
    private final double[] color;

    public StrikeOutAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "StrikeOut";
    }

    public double[] getColor() {
        return color;
    }
}
