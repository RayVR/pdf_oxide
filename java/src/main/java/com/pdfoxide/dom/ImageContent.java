package com.pdfoxide.dom;

/**
 * Image content model for PDF pages.
 */
public final class ImageContent {
    private final double x;
    private final double y;
    private final double width;
    private final double height;
    private final String path;

    private ImageContent(Builder builder) {
        this.x = builder.x;
        this.y = builder.y;
        this.width = builder.width;
        this.height = builder.height;
        this.path = builder.path;
    }

    public static Builder builder() {
        return new Builder();
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    public String getPath() {
        return path;
    }

    public static final class Builder {
        private double x;
        private double y;
        private double width;
        private double height;
        private String path;

        public Builder position(double x, double y) {
            this.x = x;
            this.y = y;
            return this;
        }

        public Builder width(double width) {
            this.width = width;
            return this;
        }

        public Builder height(double height) {
            this.height = height;
            return this;
        }

        public Builder path(String path) {
            this.path = path;
            return this;
        }

        public ImageContent build() {
            return new ImageContent(this);
        }
    }
}
