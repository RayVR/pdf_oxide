package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;

/**
 * Strongly-typed image element in a PDF page.
 *
 * <p>Image elements represent embedded images with format information, dimensions,
 * resolution metadata, and alternative text for accessibility.
 *
 * @since 1.0.0
 */
public final class PdfImage extends PdfElement {
    private final ElementId id;
    private final Rect bbox;
    private final int width;
    private final int height;
    private final String format;
    private String altText;

    // Resolution information (v0.3.1+)
    private final Float horizontalDpi;
    private final Float verticalDpi;
    private final boolean isGrayscale;

    PdfImage(ElementId id, Rect bbox, int width, int height, String format,
            String altText, Float horizontalDpi, Float verticalDpi, boolean isGrayscale) {
        this.id = id;
        this.bbox = bbox;
        this.width = width;
        this.height = height;
        this.format = format;
        this.altText = altText;
        this.horizontalDpi = horizontalDpi;
        this.verticalDpi = verticalDpi;
        this.isGrayscale = isGrayscale;
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
     * Gets the image width in pixels.
     *
     * @return width
     */
    public int getWidth() {
        return width;
    }

    /**
     * Gets the image height in pixels.
     *
     * @return height
     */
    public int getHeight() {
        return height;
    }

    /**
     * Gets the image format.
     *
     * @return format string (e.g., "JPEG", "PNG")
     */
    public String getFormat() {
        return format;
    }

    /**
     * Gets the aspect ratio (width / height).
     *
     * @return aspect ratio
     */
    public float getAspectRatio() {
        return (float) width / height;
    }

    /**
     * Gets the alternative text (for accessibility).
     *
     * @return alt text or null if not set
     */
    public String getAltText() {
        return altText;
    }

    /**
     * Sets the alternative text.
     *
     * @param altText alternative text
     */
    public void setAltText(String altText) {
        this.altText = altText;
    }

    /**
     * Checks if this image is grayscale.
     *
     * @return true if grayscale
     */
    public boolean isGrayscale() {
        return isGrayscale;
    }

    /**
     * Gets the horizontal DPI (dots per inch).
     *
     * @return horizontal DPI or null if unknown
     */
    public Float getHorizontalDpi() {
        return horizontalDpi;
    }

    /**
     * Gets the vertical DPI (dots per inch).
     *
     * @return vertical DPI or null if unknown
     */
    public Float getVerticalDpi() {
        return verticalDpi;
    }

    /**
     * Checks if this image is high resolution (>= 300 DPI).
     *
     * @return true if high resolution
     */
    public boolean isHighResolution() {
        return horizontalDpi != null && horizontalDpi >= 300;
    }

    /**
     * Checks if this image is medium resolution (>= 150 and < 300 DPI).
     *
     * @return true if medium resolution
     */
    public boolean isMediumResolution() {
        return horizontalDpi != null && horizontalDpi >= 150 && horizontalDpi < 300;
    }

    /**
     * Checks if this image is low resolution (< 150 DPI).
     *
     * @return true if low resolution
     */
    public boolean isLowResolution() {
        return horizontalDpi != null && horizontalDpi < 150;
    }

    @Override
    public String toString() {
        String dpiInfo = horizontalDpi != null ?
            String.format(", %.0f DPI", horizontalDpi) : "";
        return String.format("PdfImage(id=%s, %dx%d %s%s)",
                             id, width, height, format, dpiInfo);
    }
}
