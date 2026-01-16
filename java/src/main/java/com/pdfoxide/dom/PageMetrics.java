package com.pdfoxide.dom;

/**
 * Page size and metrics information.
 */
public final class PageMetrics {
    private final double width;
    private final double height;
    private final int rotation;

    public PageMetrics(double width, double height, int rotation) {
        this.width = width;
        this.height = height;
        this.rotation = rotation;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    public int getRotation() {
        return rotation;
    }
}
