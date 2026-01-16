package com.pdfoxide.dom;

/**
 * Text style information.
 */
public final class TextStyle {
    private final String fontName;
    private final double fontSize;
    private final boolean bold;
    private final boolean italic;
    private final double[] color;

    public TextStyle(String fontName, double fontSize, boolean bold, boolean italic, double[] color) {
        this.fontName = fontName;
        this.fontSize = fontSize;
        this.bold = bold;
        this.italic = italic;
        this.color = color;
    }

    public String getFontName() {
        return fontName;
    }

    public double getFontSize() {
        return fontSize;
    }

    public boolean isBold() {
        return bold;
    }

    public boolean isItalic() {
        return italic;
    }

    public double[] getColor() {
        return color;
    }
}
