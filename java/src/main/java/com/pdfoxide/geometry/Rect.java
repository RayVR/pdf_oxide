package com.pdfoxide.geometry;

/**
 * Immutable rectangle with floating-point coordinates.
 *
 * <p>Represents a bounding box in PDF coordinate space where (0,0) is at bottom-left.
 *
 * @since 1.0.0
 */
public final class Rect {
    private final float x;
    private final float y;
    private final float width;
    private final float height;

    /**
     * Creates a rectangle with the specified coordinates and dimensions.
     *
     * @param x left coordinate
     * @param y bottom coordinate
     * @param width width in PDF units
     * @param height height in PDF units
     */
    public Rect(float x, float y, float width, float height) {
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }

    /**
     * Gets the left coordinate.
     *
     * @return left x-coordinate
     */
    public float getX() {
        return x;
    }

    /**
     * Gets the bottom coordinate.
     *
     * @return bottom y-coordinate
     */
    public float getY() {
        return y;
    }

    /**
     * Gets the width.
     *
     * @return width in PDF units
     */
    public float getWidth() {
        return width;
    }

    /**
     * Gets the height.
     *
     * @return height in PDF units
     */
    public float getHeight() {
        return height;
    }

    /**
     * Gets the right coordinate (x + width).
     *
     * @return right x-coordinate
     */
    public float getRight() {
        return x + width;
    }

    /**
     * Gets the top coordinate (y + height).
     *
     * @return top y-coordinate
     */
    public float getTop() {
        return y + height;
    }

    /**
     * Gets the center point of the rectangle.
     *
     * @return center point
     */
    public Point getCenter() {
        return new Point(x + width / 2.0f, y + height / 2.0f);
    }

    /**
     * Checks if this rectangle contains the specified point.
     *
     * @param point point to check
     * @return true if point is within bounds
     */
    public boolean contains(Point point) {
        return point.getX() >= x && point.getX() <= x + width &&
               point.getY() >= y && point.getY() <= y + height;
    }

    /**
     * Checks if this rectangle intersects with another rectangle.
     *
     * @param other rectangle to check intersection with
     * @return true if rectangles intersect
     */
    public boolean intersects(Rect other) {
        return !(x + width < other.x || x > other.x + other.width ||
                 y + height < other.y || y > other.y + other.height);
    }

    /**
     * Checks if this rectangle completely contains another rectangle.
     *
     * @param other rectangle to check
     * @return true if other is completely contained
     */
    public boolean contains(Rect other) {
        return x <= other.x && y <= other.y &&
               x + width >= other.x + other.width &&
               y + height >= other.y + other.height;
    }

    @Override
    public boolean equals(Object obj) {
        if (!(obj instanceof Rect)) {
            return false;
        }
        Rect other = (Rect) obj;
        return Float.compare(x, other.x) == 0 &&
               Float.compare(y, other.y) == 0 &&
               Float.compare(width, other.width) == 0 &&
               Float.compare(height, other.height) == 0;
    }

    @Override
    public int hashCode() {
        int result = Float.floatToIntBits(x);
        result = 31 * result + Float.floatToIntBits(y);
        result = 31 * result + Float.floatToIntBits(width);
        result = 31 * result + Float.floatToIntBits(height);
        return result;
    }

    @Override
    public String toString() {
        return String.format("Rect(x=%.1f, y=%.1f, width=%.1f, height=%.1f)", x, y, width, height);
    }
}
