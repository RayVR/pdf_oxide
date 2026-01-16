package com.pdfoxide.geometry;

/**
 * Immutable 2D point with floating-point coordinates.
 *
 * @since 1.0.0
 */
public final class Point {
    private final float x;
    private final float y;

    /**
     * Creates a point with the specified coordinates.
     *
     * @param x x-coordinate
     * @param y y-coordinate
     */
    public Point(float x, float y) {
        this.x = x;
        this.y = y;
    }

    /**
     * Gets the x-coordinate.
     *
     * @return x value
     */
    public float getX() {
        return x;
    }

    /**
     * Gets the y-coordinate.
     *
     * @return y value
     */
    public float getY() {
        return y;
    }

    /**
     * Calculates the distance to another point.
     *
     * @param other the other point
     * @return Euclidean distance
     */
    public float distance(Point other) {
        float dx = this.x - other.x;
        float dy = this.y - other.y;
        return (float) Math.sqrt(dx * dx + dy * dy);
    }

    /**
     * Calculates the distance to a coordinate.
     *
     * @param x other x-coordinate
     * @param y other y-coordinate
     * @return Euclidean distance
     */
    public float distance(float x, float y) {
        float dx = this.x - x;
        float dy = this.y - y;
        return (float) Math.sqrt(dx * dx + dy * dy);
    }

    /**
     * Creates a new point offset from this point.
     *
     * @param dx x offset
     * @param dy y offset
     * @return new point
     */
    public Point offset(float dx, float dy) {
        return new Point(x + dx, y + dy);
    }

    @Override
    public boolean equals(Object obj) {
        if (!(obj instanceof Point)) {
            return false;
        }
        Point other = (Point) obj;
        return Float.compare(x, other.x) == 0 && Float.compare(y, other.y) == 0;
    }

    @Override
    public int hashCode() {
        int result = Float.floatToIntBits(x);
        result = 31 * result + Float.floatToIntBits(y);
        return result;
    }

    @Override
    public String toString() {
        return String.format("Point(%.1f, %.1f)", x, y);
    }
}
