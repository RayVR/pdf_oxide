package com.pdfoxide.geometry;

/**
 * Immutable RGB color with floating-point components.
 *
 * <p>Color components are in the range [0.0, 1.0] where:
 * <ul>
 *   <li>0.0 = 0% intensity (black)
 *   <li>1.0 = 100% intensity (maximum)
 * </ul>
 *
 * @since 1.0.0
 */
public final class Color {
    private final float r;
    private final float g;
    private final float b;

    /**
     * Creates a color with the specified RGB components.
     *
     * @param r red component (0.0-1.0)
     * @param g green component (0.0-1.0)
     * @param b blue component (0.0-1.0)
     */
    public Color(float r, float g, float b) {
        this.r = clamp(r);
        this.g = clamp(g);
        this.b = clamp(b);
    }

    /**
     * Creates a color from RGB values (0-255).
     *
     * @param red red component (0-255)
     * @param green green component (0-255)
     * @param blue blue component (0-255)
     * @return color with converted components
     */
    public static Color fromRGB(int red, int green, int blue) {
        return new Color(
            red / 255.0f,
            green / 255.0f,
            blue / 255.0f
        );
    }

    /**
     * Creates a color from an RGB hex value.
     *
     * @param hex hex color (e.g., 0xFF0000 for red)
     * @return color with converted components
     */
    public static Color fromHex(int hex) {
        int red = (hex >> 16) & 0xFF;
        int green = (hex >> 8) & 0xFF;
        int blue = hex & 0xFF;
        return fromRGB(red, green, blue);
    }

    /**
     * Gets the red component.
     *
     * @return red value (0.0-1.0)
     */
    public float getRed() {
        return r;
    }

    /**
     * Gets the green component.
     *
     * @return green value (0.0-1.0)
     */
    public float getGreen() {
        return g;
    }

    /**
     * Gets the blue component.
     *
     * @return blue value (0.0-1.0)
     */
    public float getBlue() {
        return b;
    }

    /**
     * Converts to RGB values (0-255).
     *
     * @return array [red, green, blue] in range 0-255
     */
    public int[] toRGB() {
        return new int[]{
            Math.round(r * 255),
            Math.round(g * 255),
            Math.round(b * 255)
        };
    }

    /**
     * Converts to hex string.
     *
     * @return hex color string (e.g., "#FF0000")
     */
    public String toHexString() {
        int[] rgb = toRGB();
        return String.format("#%02X%02X%02X", rgb[0], rgb[1], rgb[2]);
    }

    /**
     * Converts to hex integer.
     *
     * @return hex color value
     */
    public int toHex() {
        int[] rgb = toRGB();
        return (rgb[0] << 16) | (rgb[1] << 8) | rgb[2];
    }

    /**
     * Checks if this color is grayscale (R=G=B).
     *
     * @return true if all components are equal
     */
    public boolean isGrayscale() {
        return Float.compare(r, g) == 0 && Float.compare(g, b) == 0;
    }

    /**
     * Gets the brightness (luminance) of this color.
     *
     * @return brightness value (0.0-1.0)
     */
    public float getBrightness() {
        return 0.299f * r + 0.587f * g + 0.114f * b;
    }

    /**
     * Creates a new color with adjusted brightness.
     *
     * @param factor brightness factor (0.5 = darker, 1.0 = same, 2.0 = lighter)
     * @return new color with adjusted brightness
     */
    public Color adjustBrightness(float factor) {
        return new Color(r * factor, g * factor, b * factor);
    }

    /**
     * Predefined color: black.
     */
    public static final Color BLACK = new Color(0, 0, 0);

    /**
     * Predefined color: white.
     */
    public static final Color WHITE = new Color(1, 1, 1);

    /**
     * Predefined color: red.
     */
    public static final Color RED = new Color(1, 0, 0);

    /**
     * Predefined color: green.
     */
    public static final Color GREEN = new Color(0, 1, 0);

    /**
     * Predefined color: blue.
     */
    public static final Color BLUE = new Color(0, 0, 1);

    /**
     * Predefined color: yellow.
     */
    public static final Color YELLOW = new Color(1, 1, 0);

    /**
     * Predefined color: cyan.
     */
    public static final Color CYAN = new Color(0, 1, 1);

    /**
     * Predefined color: magenta.
     */
    public static final Color MAGENTA = new Color(1, 0, 1);

    private static float clamp(float value) {
        return Math.max(0.0f, Math.min(1.0f, value));
    }

    @Override
    public boolean equals(Object obj) {
        if (!(obj instanceof Color)) {
            return false;
        }
        Color other = (Color) obj;
        return Float.compare(r, other.r) == 0 &&
               Float.compare(g, other.g) == 0 &&
               Float.compare(b, other.b) == 0;
    }

    @Override
    public int hashCode() {
        int result = Float.floatToIntBits(r);
        result = 31 * result + Float.floatToIntBits(g);
        result = 31 * result + Float.floatToIntBits(b);
        return result;
    }

    @Override
    public String toString() {
        return String.format("Color(%.2f, %.2f, %.2f)", r, g, b);
    }
}
