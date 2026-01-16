package com.pdfoxide.dom;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Point;
import com.pdfoxide.geometry.Rect;

/**
 * Strongly-typed text element in a PDF page with DOM editing capabilities.
 *
 * <p>Text elements represent text runs with font information, positioning, and styling.
 * They support transformation properties (rotation, translation) and style modifications.
 *
 * @since 1.0.0
 */
public final class PdfText extends PdfElement {
    private final ElementId id;
    private final Rect bbox;
    private final String fontName;
    private final float fontSize;
    private final boolean bold;
    private final boolean italic;
    private final Color color;
    private String text;

    // Transformation properties
    private final Point origin;
    private final Float rotationDegrees;

    PdfText(ElementId id, String text, Rect bbox, String fontName, float fontSize,
            boolean bold, boolean italic, Color color, Point origin, Float rotationDegrees) {
        this.id = id;
        this.text = text;
        this.bbox = bbox;
        this.fontName = fontName;
        this.fontSize = fontSize;
        this.bold = bold;
        this.italic = italic;
        this.color = color;
        this.origin = origin;
        this.rotationDegrees = rotationDegrees;
    }

    @Override
    public ElementId getId() {
        return id;
    }

    @Override
    public Rect getBbox() {
        return bbox;
    }

    /**
     * Gets the text content.
     *
     * @return text string
     */
    public String getText() {
        return text;
    }

    /**
     * Sets the text content.
     *
     * @param newText new text value
     */
    public void setText(String newText) {
        this.text = newText;
    }

    /**
     * Gets the font name.
     *
     * @return font name (e.g., "Helvetica", "Times-Roman")
     */
    public String getFontName() {
        return fontName;
    }

    /**
     * Gets the font size in points.
     *
     * @return font size
     */
    public float getFontSize() {
        return fontSize;
    }

    /**
     * Checks if the text is bold.
     *
     * @return true if bold
     */
    public boolean isBold() {
        return bold;
    }

    /**
     * Checks if the text is italic.
     *
     * @return true if italic
     */
    public boolean isItalic() {
        return italic;
    }

    /**
     * Gets the text color.
     *
     * @return RGB color
     */
    public Color getColor() {
        return color;
    }

    /**
     * Gets the baseline origin point if available.
     *
     * @return origin point or null if not set
     */
    public Point getOrigin() {
        return origin;
    }

    /**
     * Gets the rotation angle in degrees if available.
     *
     * @return rotation in degrees or null if not rotated
     */
    public Float getRotationDegrees() {
        return rotationDegrees;
    }

    /**
     * Checks if this text is rotated.
     *
     * @return true if rotation is non-zero
     */
    public boolean isRotated() {
        return rotationDegrees != null && rotationDegrees != 0.0f;
    }

    /**
     * Appends text to the current content.
     *
     * @param suffix text to append
     */
    public void append(String suffix) {
        this.text += suffix;
    }

    /**
     * Replaces all occurrences of a substring.
     *
     * @param oldText text to replace
     * @param newText replacement text
     * @return number of replacements made
     */
    public int replace(String oldText, String newText) {
        int count = countOccurrences(this.text, oldText);
        this.text = this.text.replace(oldText, newText);
        return count;
    }

    /**
     * Clears the text content.
     */
    public void clear() {
        this.text = "";
    }

    /**
     * Checks if the text is empty.
     *
     * @return true if length is 0
     */
    public boolean isEmpty() {
        return text.isEmpty();
    }

    /**
     * Gets the length in characters.
     *
     * @return text length
     */
    public int length() {
        return text.length();
    }

    /**
     * Checks if the text contains a substring.
     *
     * @param needle substring to find
     * @return true if found
     */
    public boolean contains(String needle) {
        return text.contains(needle);
    }

    /**
     * Checks if the text starts with a prefix.
     *
     * @param prefix prefix to check
     * @return true if starts with prefix
     */
    public boolean startsWith(String prefix) {
        return text.startsWith(prefix);
    }

    /**
     * Checks if the text ends with a suffix.
     *
     * @param suffix suffix to check
     * @return true if ends with suffix
     */
    public boolean endsWith(String suffix) {
        return text.endsWith(suffix);
    }

    private static int countOccurrences(String string, String substring) {
        int count = 0;
        int index = 0;
        while ((index = string.indexOf(substring, index)) != -1) {
            count++;
            index += substring.length();
        }
        return count;
    }

    @Override
    public String toString() {
        return String.format("PdfText(id=%s, text='%s', font=%s, size=%.1f)",
                             id, text, fontName, fontSize);
    }
}
