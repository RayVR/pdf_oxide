package com.pdfoxide.geometry;

/**
 * Width and height dimensions.
 */
public final class Dimensions {
    private final double width;
    private final double height;

    public Dimensions(double width, double height) {
        this.width = width;
        this.height = height;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    @Override
    public String toString() {
        return "Dimensions{" +
                "width=" + width +
                ", height=" + height +
                '}';
    }
}
