package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;
import com.pdfoxide.internal.NativeHandle;

import java.util.ArrayList;
import java.util.List;

/**
 * Represents a PDF page with DOM-like element navigation and editing.
 *
 * <p>Pages contain content elements (text, images, paths, tables) that can be
 * queried, searched, and modified. Elements maintain their IDs for reference
 * across operations.
 *
 * @since 1.0.0
 */
public final class PdfPage implements AutoCloseable {
    private final NativeHandle handle;
    private final int pageIndex;
    private final float width;
    private final float height;
    private boolean closed = false;

    /**
     * Creates a PdfPage wrapper around a native handle.
     *
     * @param handle native pointer wrapper
     * @param pageIndex zero-based page index
     * @param width page width in PDF units
     * @param height page height in PDF units
     */
    PdfPage(NativeHandle handle, int pageIndex, float width, float height) {
        this.handle = handle;
        this.pageIndex = pageIndex;
        this.width = width;
        this.height = height;
    }

    /**
     * Gets the page index (zero-based).
     *
     * @return page number
     */
    public int getPageIndex() {
        return pageIndex;
    }

    /**
     * Gets the page width in PDF units.
     *
     * @return width (typically in points, where 72 points = 1 inch)
     */
    public float getWidth() {
        return width;
    }

    /**
     * Gets the page height in PDF units.
     *
     * @return height (typically in points)
     */
    public float getHeight() {
        return height;
    }

    /**
     * Gets the bounding box of the entire page.
     *
     * @return rectangle from (0,0) to (width, height)
     */
    public Rect getBounds() {
        return new Rect(0, 0, width, height);
    }

    /**
     * Gets the native pointer (for JNI operations).
     *
     * @return native handle pointer
     */
    protected long getHandle() {
        ensureNotClosed();
        return handle.ptr();
    }

    /**
     * Gets all top-level child elements on the page.
     *
     * @return list of all elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfElement> getChildren() {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all text elements containing the specified substring.
     *
     * @param needle text to search for
     * @return list of matching text elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfText> findTextContaining(String needle) {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all text elements matching a predicate.
     *
     * <p>This is a convenience method for common search patterns.
     *
     * @param startsWith if non-null, find text starting with this prefix
     * @param endsWith if non-null, find text ending with this suffix
     * @param contains if non-null, find text containing this substring
     * @return list of matching text elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfText> findText(String startsWith, String endsWith, String contains) {
        List<PdfText> results = new ArrayList<>();

        List<PdfText> all = getAllText();
        for (PdfText text : all) {
            boolean matches = true;
            if (startsWith != null && !text.getText().startsWith(startsWith)) {
                matches = false;
            }
            if (endsWith != null && !text.getText().endsWith(endsWith)) {
                matches = false;
            }
            if (contains != null && !text.getText().contains(contains)) {
                matches = false;
            }
            if (matches) {
                results.add(text);
            }
        }

        return results;
    }

    /**
     * Gets all text elements on the page.
     *
     * @return list of all text elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfText> getAllText() {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all image elements.
     *
     * @return list of all image elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfImage> findImages() {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all image elements in a specific region.
     *
     * @param region bounding box to search within
     * @return list of images in the region
     * @throws IllegalStateException if page is closed
     */
    public List<PdfImage> findImagesInRegion(Rect region) {
        ensureNotClosed();
        List<PdfImage> results = new ArrayList<>();

        for (PdfImage img : findImages()) {
            if (region.intersects(img.getBbox())) {
                results.add(img);
            }
        }

        return results;
    }

    /**
     * Finds all path/graphics elements.
     *
     * @return list of all path elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfPath> findPaths() {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all table elements.
     *
     * @return list of all table elements
     * @throws IllegalStateException if page is closed
     */
    public List<PdfTable> findTables() {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Finds all elements in a specific region.
     *
     * @param region bounding box to search within
     * @return list of all elements in the region
     * @throws IllegalStateException if page is closed
     */
    public List<PdfElement> findInRegion(Rect region) {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return new ArrayList<>();
    }

    /**
     * Sets the text content of a specific element.
     *
     * @param id element ID
     * @param text new text value
     * @throws IllegalStateException if page is closed
     */
    public void setText(ElementId id, String text) {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
    }

    /**
     * Removes an element from the page.
     *
     * @param id element ID
     * @throws IllegalStateException if page is closed
     */
    public void removeElement(ElementId id) {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
    }

    /**
     * Adds a new text element to the page.
     *
     * @param text text content
     * @param x x-coordinate
     * @param y y-coordinate
     * @param fontSize font size
     * @param fontName font name
     * @return ID of the newly created element
     * @throws IllegalStateException if page is closed
     */
    public ElementId addText(String text, float x, float y, float fontSize, String fontName) {
        ensureNotClosed();
        // Placeholder - will be implemented via JNI
        return ElementId.generate();
    }

    /**
     * Checks if the page is closed.
     *
     * @return true if page has been closed
     */
    public boolean isClosed() {
        return closed;
    }

    @Override
    public void close() {
        if (!closed) {
            handle.close();
            closed = true;
        }
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("PdfPage has been closed");
        }
    }

    @Override
    public String toString() {
        return String.format("PdfPage(index=%d, %.1f×%.1f)", pageIndex, width, height);
    }
}
