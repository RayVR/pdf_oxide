package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;

/**
 * Abstract base for PDF page elements that can be returned from element queries.
 *
 * <p>Elements can be of different types (text, image, path, table, structure).
 * Use the type-checking methods or type-casting methods to work with specific types.
 *
 * @since 1.0.0
 */
public abstract class PdfElement {

    /**
     * Gets the element ID.
     *
     * @return unique identifier
     */
    public abstract ElementId getId();

    /**
     * Gets the bounding box.
     *
     * @return rectangle bounds in PDF coordinates
     */
    public abstract Rect getBbox();

    /**
     * Checks if this is a text element.
     *
     * @return true if this is a PdfText
     */
    public boolean isText() {
        return this instanceof PdfText;
    }

    /**
     * Checks if this is an image element.
     *
     * @return true if this is a PdfImage
     */
    public boolean isImage() {
        return this instanceof PdfImage;
    }

    /**
     * Checks if this is a path element.
     *
     * @return true if this is a PdfPath
     */
    public boolean isPath() {
        return this instanceof PdfPath;
    }

    /**
     * Checks if this is a table element.
     *
     * @return true if this is a PdfTable
     */
    public boolean isTable() {
        return this instanceof PdfTable;
    }

    /**
     * Checks if this is a structure element.
     *
     * @return true if this is a PdfStructure
     */
    public boolean isStructure() {
        return this instanceof PdfStructure;
    }

    /**
     * Casts to PdfText if this is a text element.
     *
     * @return this as PdfText, or null if not a text element
     */
    public PdfText asText() {
        return isText() ? (PdfText) this : null;
    }

    /**
     * Casts to PdfImage if this is an image element.
     *
     * @return this as PdfImage, or null if not an image element
     */
    public PdfImage asImage() {
        return isImage() ? (PdfImage) this : null;
    }

    /**
     * Casts to PdfPath if this is a path element.
     *
     * @return this as PdfPath, or null if not a path element
     */
    public PdfPath asPath() {
        return isPath() ? (PdfPath) this : null;
    }

    /**
     * Casts to PdfTable if this is a table element.
     *
     * @return this as PdfTable, or null if not a table element
     */
    public PdfTable asTable() {
        return isTable() ? (PdfTable) this : null;
    }

    /**
     * Casts to PdfStructure if this is a structure element.
     *
     * @return this as PdfStructure, or null if not a structure element
     */
    public PdfStructure asStructure() {
        return isStructure() ? (PdfStructure) this : null;
    }

    /**
     * Factory method to create a text element.
     *
     * @param id element ID
     * @param text text content
     * @param bbox bounding box
     * @param fontName font name
     * @param fontSize font size
     * @param bold bold flag
     * @param italic italic flag
     * @param color text color
     * @param origin origin point (optional)
     * @param rotationDegrees rotation angle (optional)
     * @return new PdfText element
     */
    public static PdfElement ofText(ElementId id, String text, Rect bbox,
                                    String fontName, float fontSize, boolean bold,
                                    boolean italic, com.pdfoxide.geometry.Color color,
                                    com.pdfoxide.geometry.Point origin, Float rotationDegrees) {
        return new PdfText(id, text, bbox, fontName, fontSize, bold, italic, color, origin, rotationDegrees);
    }

    /**
     * Factory method to create an image element.
     *
     * @param id element ID
     * @param bbox bounding box
     * @param width image width in pixels
     * @param height image height in pixels
     * @param format image format
     * @param altText alternative text
     * @param horizontalDpi horizontal resolution
     * @param verticalDpi vertical resolution
     * @param isGrayscale grayscale flag
     * @return new PdfImage element
     */
    public static PdfElement ofImage(ElementId id, Rect bbox, int width, int height,
                                    String format, String altText,
                                    Float horizontalDpi, Float verticalDpi, boolean isGrayscale) {
        return new PdfImage(id, bbox, width, height, format, altText, horizontalDpi, verticalDpi, isGrayscale);
    }

    /**
     * Factory method to create a path element.
     *
     * @param id element ID
     * @param bbox bounding box
     * @param strokeColor stroke color
     * @param fillColor fill color
     * @param strokeWidth stroke width
     * @param description path description
     * @return new PdfPath element
     */
    public static PdfElement ofPath(ElementId id, Rect bbox,
                                   com.pdfoxide.geometry.Color strokeColor,
                                   com.pdfoxide.geometry.Color fillColor,
                                   float strokeWidth, String description) {
        return new PdfPath(id, bbox, strokeColor, fillColor, strokeWidth, description);
    }

    /**
     * Factory method to create a table element.
     *
     * @param id element ID
     * @param bbox bounding box
     * @param rows row count
     * @param cols column count
     * @param hasHeader header flag
     * @param caption table caption
     * @param detectionConfidence detection confidence
     * @param fromStructureTree structure tree flag
     * @param cells cell contents
     * @return new PdfTable element
     */
    public static PdfElement ofTable(ElementId id, Rect bbox, int rows, int cols,
                                    boolean hasHeader, String caption,
                                    float detectionConfidence, boolean fromStructureTree,
                                    java.util.List<java.util.List<String>> cells) {
        return new PdfTable(id, bbox, rows, cols, hasHeader, caption, detectionConfidence, fromStructureTree, cells);
    }

    /**
     * Factory method to create a structure element.
     *
     * @param id element ID
     * @param structureType structure type
     * @param bbox bounding box
     * @return new PdfStructure element
     */
    public static PdfElement ofStructure(ElementId id, String structureType, Rect bbox) {
        return new PdfStructure(id, structureType, bbox);
    }
}
