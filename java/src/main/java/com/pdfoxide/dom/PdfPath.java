package com.pdfoxide.dom;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Rect;

/**
 * Strongly-typed path/graphics element in a PDF page.
 *
 * <p>Path elements represent vector graphics including lines, curves, rectangles,
 * and other shapes with stroke and fill properties.
 *
 * @since 1.0.0
 */
public final class PdfPath extends PdfElement {
    private final ElementId id;
    private final Rect bbox;
    private Color strokeColor;
    private Color fillColor;
    private float strokeWidth;
    private final String description;  // For complex path data

    PdfPath(ElementId id, Rect bbox, Color strokeColor, Color fillColor,
            float strokeWidth, String description) {
        this.id = id;
        this.bbox = bbox;
        this.strokeColor = strokeColor;
        this.fillColor = fillColor;
        this.strokeWidth = strokeWidth;
        this.description = description;
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
     * Gets the stroke color.
     *
     * @return stroke color or null if no stroke
     */
    public Color getStrokeColor() {
        return strokeColor;
    }

    /**
     * Sets the stroke color.
     *
     * @param color new stroke color
     */
    public void setStrokeColor(Color color) {
        this.strokeColor = color;
    }

    /**
     * Gets the fill color.
     *
     * @return fill color or null if no fill
     */
    public Color getFillColor() {
        return fillColor;
    }

    /**
     * Sets the fill color.
     *
     * @param color new fill color
     */
    public void setFillColor(Color color) {
        this.fillColor = color;
    }

    /**
     * Gets the stroke width.
     *
     * @return stroke width in PDF units
     */
    public float getStrokeWidth() {
        return strokeWidth;
    }

    /**
     * Sets the stroke width.
     *
     * @param width new stroke width
     */
    public void setStrokeWidth(float width) {
        this.strokeWidth = width;
    }

    /**
     * Checks if this path has a stroke.
     *
     * @return true if stroke color is set and width > 0
     */
    public boolean hasStroke() {
        return strokeColor != null && strokeWidth > 0;
    }

    /**
     * Checks if this path has a fill.
     *
     * @return true if fill color is set
     */
    public boolean hasFill() {
        return fillColor != null;
    }

    /**
     * Converts this path to an SVG path element string.
     *
     * @return SVG path element
     */
    public String toSvgPath() {
        StringBuilder svg = new StringBuilder("<path d=\"");
        svg.append(description);
        svg.append("\"");

        if (hasStroke() && strokeColor != null) {
            int[] rgb = strokeColor.toRGB();
            svg.append(String.format(" stroke=\"rgb(%d,%d,%d)\"", rgb[0], rgb[1], rgb[2]));
            svg.append(String.format(" stroke-width=\"%.1f\"", strokeWidth));
        } else {
            svg.append(" stroke=\"none\"");
        }

        if (hasFill() && fillColor != null) {
            int[] rgb = fillColor.toRGB();
            svg.append(String.format(" fill=\"rgb(%d,%d,%d)\"", rgb[0], rgb[1], rgb[2]));
        } else {
            svg.append(" fill=\"none\"");
        }

        svg.append("/>");
        return svg.toString();
    }

    /**
     * Converts this path to a complete SVG document.
     *
     * @return SVG document string
     */
    public String toSvgDocument() {
        return String.format(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n" +
            "<svg xmlns=\"http://www.w3.org/2000/svg\" " +
            "viewBox=\"%.1f %.1f %.1f %.1f\" width=\"%.1f\" height=\"%.1f\">\n" +
            "  %s\n" +
            "</svg>",
            bbox.getX(), bbox.getY(), bbox.getWidth(), bbox.getHeight(),
            bbox.getWidth(), bbox.getHeight(),
            toSvgPath()
        );
    }

    @Override
    public String toString() {
        return String.format("PdfPath(id=%s, stroke=%s, fill=%s)",
                             id, hasStroke() ? "yes" : "no", hasFill() ? "yes" : "no");
    }
}
