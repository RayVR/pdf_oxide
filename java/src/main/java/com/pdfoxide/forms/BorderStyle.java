package com.pdfoxide.forms;

/**
 * Border style for form field widgets.
 *
 * @since 1.0.0
 */
public final class BorderStyle {
    private final BorderStyleType style;
    private final double width;
    private final int[] dashPattern;

    public enum BorderStyleType {
        SOLID("S"),
        BEVELED("B"),
        DASHED("D"),
        INSET("I"),
        UNDERLINE("U");

        private final String code;

        BorderStyleType(String code) {
            this.code = code;
        }

        public String getCode() {
            return code;
        }
    }

    /**
     * Constructs a border style.
     *
     * @param style border appearance
     * @param width border width in points
     */
    public BorderStyle(BorderStyleType style, double width) {
        this(style, width, null);
    }

    /**
     * Constructs a dashed border style.
     *
     * @param width border width in points
     * @param dashPattern dash pattern array
     */
    public BorderStyle(double width, int[] dashPattern) {
        this(BorderStyleType.DASHED, width, dashPattern);
    }

    /**
     * Constructs a fully configured border style.
     *
     * @param style border appearance
     * @param width border width in points
     * @param dashPattern dash pattern for dashed borders (optional)
     */
    public BorderStyle(BorderStyleType style, double width, int[] dashPattern) {
        this.style = style;
        this.width = width;
        this.dashPattern = dashPattern;
    }

    /**
     * Gets the border style.
     *
     * @return style
     */
    public BorderStyleType getStyle() {
        return style;
    }

    /**
     * Gets the border width.
     *
     * @return width in points
     */
    public double getWidth() {
        return width;
    }

    /**
     * Gets the dash pattern.
     *
     * @return dash pattern array, empty if solid
     */
    public int[] getDashPattern() {
        return dashPattern != null ? dashPattern.clone() : new int[0];
    }

    /**
     * Creates a solid border.
     *
     * @param width border width
     * @return border style
     */
    public static BorderStyle solid(double width) {
        return new BorderStyle(BorderStyleType.SOLID, width);
    }

    /**
     * Creates a beveled border.
     *
     * @param width border width
     * @return border style
     */
    public static BorderStyle beveled(double width) {
        return new BorderStyle(BorderStyleType.BEVELED, width);
    }

    /**
     * Creates a dashed border.
     *
     * @param width border width
     * @param dashPattern dash pattern
     * @return border style
     */
    public static BorderStyle dashed(double width, int[] dashPattern) {
        return new BorderStyle(width, dashPattern);
    }

    /**
     * Creates an inset border.
     *
     * @param width border width
     * @return border style
     */
    public static BorderStyle inset(double width) {
        return new BorderStyle(BorderStyleType.INSET, width);
    }

    /**
     * Creates an underline border.
     *
     * @param width border width
     * @return border style
     */
    public static BorderStyle underline(double width) {
        return new BorderStyle(BorderStyleType.UNDERLINE, width);
    }

    @Override
    public String toString() {
        return String.format("BorderStyle(%s, width=%.1f)", style, width);
    }
}
