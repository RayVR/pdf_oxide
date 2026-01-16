package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Point;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Polygon annotation for drawing closed polygonal shapes.
 *
 * <p>Polygon annotations draw closed shapes connecting multiple points,
 * with optional fill and stroke styling.
 *
 * @since 1.0.0
 */
public final class PolygonAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final List<Point> vertices;
    private final Optional<Color> strokeColor;
    private final Optional<Color> fillColor;
    private final double strokeWidth;
    private final Optional<Integer> strokeDashPattern;

    /**
     * Constructs a polygon annotation.
     *
     * @param rect bounding box
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param vertices polygon vertices (at least 3 points)
     * @param strokeColor outline color (optional)
     * @param fillColor fill color (optional)
     * @param strokeWidth outline width in points
     * @param strokeDashPattern dash pattern (optional)
     */
    private PolygonAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            List<Point> vertices,
            Optional<Color> strokeColor,
            Optional<Color> fillColor,
            double strokeWidth,
            Optional<Integer> strokeDashPattern) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.vertices = vertices;
        this.strokeColor = strokeColor;
        this.fillColor = fillColor;
        this.strokeWidth = strokeWidth;
        this.strokeDashPattern = strokeDashPattern;
    }

    @Override
    public String getType() {
        return "Polygon";
    }

    @Override
    public Rect getRect() {
        return rect;
    }

    @Override
    public String getContents() {
        return contents;
    }

    @Override
    public Optional<String> getAuthor() {
        return author;
    }

    @Override
    public Optional<Instant> getCreatedDate() {
        return createdDate;
    }

    @Override
    public Optional<Instant> getModifiedDate() {
        return modifiedDate;
    }

    @Override
    public Optional<String> getSubject() {
        return subject;
    }

    @Override
    public int getFlags() {
        return flags;
    }

    /**
     * Gets the polygon vertices.
     *
     * @return list of vertices (immutable)
     */
    public List<Point> getVertices() {
        return Collections.unmodifiableList(vertices);
    }

    /**
     * Gets the number of vertices.
     *
     * @return vertex count
     */
    public int getVertexCount() {
        return vertices.size();
    }

    /**
     * Gets the stroke (outline) color.
     *
     * @return color, empty for no stroke
     */
    public Optional<Color> getStrokeColor() {
        return strokeColor;
    }

    /**
     * Gets the fill color.
     *
     * @return color, empty for no fill
     */
    public Optional<Color> getFillColor() {
        return fillColor;
    }

    /**
     * Gets the stroke width.
     *
     * @return width in points
     */
    public double getStrokeWidth() {
        return strokeWidth;
    }

    /**
     * Gets the stroke dash pattern.
     *
     * @return dash pattern (0=solid, 1=dashed, etc.), empty for solid
     */
    public Optional<Integer> getStrokeDashPattern() {
        return strokeDashPattern;
    }

    /**
     * Creates a builder for polygon annotations.
     *
     * @param vertices polygon points (at least 3)
     * @return builder
     */
    public static Builder builder(List<Point> vertices) {
        return new Builder(vertices);
    }

    /**
     * Fluent builder for polygon annotations.
     */
    public static final class Builder {
        private final List<Point> vertices;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Color> strokeColor = Optional.empty();
        private Optional<Color> fillColor = Optional.empty();
        private double strokeWidth = 1.0;
        private Optional<Integer> strokeDashPattern = Optional.empty();

        private Builder(List<Point> vertices) {
            if (vertices.size() < 3) {
                throw new IllegalArgumentException("Polygon must have at least 3 vertices");
            }
            this.vertices = vertices;
        }

        public Builder contents(String contents) {
            this.contents = contents;
            return this;
        }

        public Builder author(String author) {
            this.author = Optional.of(author);
            return this;
        }

        public Builder createdDate(Instant createdDate) {
            this.createdDate = Optional.of(createdDate);
            return this;
        }

        public Builder modifiedDate(Instant modifiedDate) {
            this.modifiedDate = Optional.of(modifiedDate);
            return this;
        }

        public Builder subject(String subject) {
            this.subject = Optional.of(subject);
            return this;
        }

        public Builder flags(int flags) {
            this.flags = flags;
            return this;
        }

        public Builder strokeColor(Color color) {
            this.strokeColor = Optional.of(color);
            return this;
        }

        public Builder fillColor(Color color) {
            this.fillColor = Optional.of(color);
            return this;
        }

        public Builder strokeWidth(double width) {
            this.strokeWidth = width;
            return this;
        }

        public Builder strokeDashPattern(int pattern) {
            this.strokeDashPattern = Optional.of(pattern);
            return this;
        }

        public PolygonAnnotation build() {
            // Calculate bounding box from vertices
            double minX = Double.MAX_VALUE, minY = Double.MAX_VALUE;
            double maxX = Double.MIN_VALUE, maxY = Double.MIN_VALUE;

            for (Point v : vertices) {
                minX = Math.min(minX, v.getX());
                minY = Math.min(minY, v.getY());
                maxX = Math.max(maxX, v.getX());
                maxY = Math.max(maxY, v.getY());
            }

            Rect rect = new Rect((float)minX, (float)minY, (float)(maxX - minX), (float)(maxY - minY));

            return new PolygonAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                vertices,
                strokeColor,
                fillColor,
                strokeWidth,
                strokeDashPattern
            );
        }
    }

    @Override
    public String toString() {
        return String.format("PolygonAnnotation(%d vertices, rect=%s)", getVertexCount(), rect);
    }
}
