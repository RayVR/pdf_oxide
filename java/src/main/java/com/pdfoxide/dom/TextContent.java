package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;

/**
 * Text content model for PDF pages.
 */
public final class TextContent {
    private final String text;
    private final double x;
    private final double y;
    private final double fontSize;
    private final String fontName;
    private final double[] color;

    private TextContent(Builder builder) {
        this.text = builder.text;
        this.x = builder.x;
        this.y = builder.y;
        this.fontSize = builder.fontSize;
        this.fontName = builder.fontName;
        this.color = builder.color;
    }

    public static Builder builder() {
        return new Builder();
    }

    public String getText() {
        return text;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getFontSize() {
        return fontSize;
    }

    public String getFontName() {
        return fontName;
    }

    public double[] getColor() {
        return color;
    }

    public static final class Builder {
        private String text;
        private double x;
        private double y;
        private double fontSize = 12.0;
        private String fontName = "Helvetica";
        private double[] color = new double[]{0.0, 0.0, 0.0};

        public Builder text(String text) {
            this.text = text;
            return this;
        }

        public Builder position(double x, double y) {
            this.x = x;
            this.y = y;
            return this;
        }

        public Builder fontSize(double fontSize) {
            this.fontSize = fontSize;
            return this;
        }

        public Builder fontName(String fontName) {
            this.fontName = fontName;
            return this;
        }

        public Builder color(double r, double g, double b) {
            this.color = new double[]{r, g, b};
            return this;
        }

        public TextContent build() {
            return new TextContent(this);
        }
    }
}
