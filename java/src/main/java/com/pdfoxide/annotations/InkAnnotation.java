package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Point;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Ink annotation for freehand drawings and annotations.
 *
 * <p>Ink annotations display freehand drawn paths, like signatures or sketches.
 * Each annotation can contain multiple separate ink strokes.
 *
 * @since 1.0.0
 */
public final class InkAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final List<List<Point>> inkPaths;
    private final Optional<Color> inkColor;
    private final double inkThickness;

    /**
     * Constructs an ink annotation.
     *
     * @param rect bounding box
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param inkPaths list of paths, each path is a list of points
     * @param inkColor ink color (optional)
     * @param inkThickness line thickness in points
     */
    private InkAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            List<List<Point>> inkPaths,
            Optional<Color> inkColor,
            double inkThickness) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.inkPaths = inkPaths;
        this.inkColor = inkColor;
        this.inkThickness = inkThickness;
    }

    @Override
    public String getType() {
        return "Ink";
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
     * Gets all ink paths/strokes.
     *
     * @return list of paths, each path is a list of points (immutable)
     */
    public List<List<Point>> getInkPaths() {
        return Collections.unmodifiableList(inkPaths);
    }

    /**
     * Gets the number of strokes.
     *
     * @return stroke count
     */
    public int getStrokeCount() {
        return inkPaths.size();
    }

    /**
     * Gets the total number of points across all strokes.
     *
     * @return total point count
     */
    public int getPointCount() {
        return inkPaths.stream().mapToInt(List::size).sum();
    }

    /**
     * Gets the ink color.
     *
     * @return ink color, default is black
     */
    public Color getInkColor() {
        return inkColor.orElse(Color.BLACK);
    }

    /**
     * Gets the ink thickness.
     *
     * @return thickness in points
     */
    public double getInkThickness() {
        return inkThickness;
    }

    /**
     * Creates a builder for ink annotations.
     *
     * @return builder
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * Fluent builder for ink annotations.
     */
    public static final class Builder {
        private final java.util.List<List<Point>> inkPaths = new java.util.ArrayList<>();
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Color> inkColor = Optional.empty();
        private double inkThickness = 1.0;

        /**
         * Adds an ink path (stroke).
         *
         * @param path list of points
         * @return this builder
         */
        public Builder addPath(List<Point> path) {
            if (path.size() < 2) {
                throw new IllegalArgumentException("Path must have at least 2 points");
            }
            inkPaths.add(path);
            return this;
        }

        /**
         * Sets the content/label.
         *
         * @param contents annotation text
         * @return this builder
         */
        public Builder contents(String contents) {
            this.contents = contents;
            return this;
        }

        /**
         * Sets the author.
         *
         * @param author creator name
         * @return this builder
         */
        public Builder author(String author) {
            this.author = Optional.of(author);
            return this;
        }

        /**
         * Sets the creation date.
         *
         * @param createdDate creation timestamp
         * @return this builder
         */
        public Builder createdDate(Instant createdDate) {
            this.createdDate = Optional.of(createdDate);
            return this;
        }

        /**
         * Sets the modification date.
         *
         * @param modifiedDate modification timestamp
         * @return this builder
         */
        public Builder modifiedDate(Instant modifiedDate) {
            this.modifiedDate = Optional.of(modifiedDate);
            return this;
        }

        /**
         * Sets the subject.
         *
         * @param subject annotation topic
         * @return this builder
         */
        public Builder subject(String subject) {
            this.subject = Optional.of(subject);
            return this;
        }

        /**
         * Sets the display flags.
         *
         * @param flags combination of AnnotationFlags constants
         * @return this builder
         */
        public Builder flags(int flags) {
            this.flags = flags;
            return this;
        }

        /**
         * Sets the ink color.
         *
         * @param color ink color
         * @return this builder
         */
        public Builder inkColor(Color color) {
            this.inkColor = Optional.of(color);
            return this;
        }

        /**
         * Sets the ink thickness.
         *
         * @param thickness line thickness in points
         * @return this builder
         */
        public Builder inkThickness(double thickness) {
            this.inkThickness = thickness;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return ink annotation
         */
        public InkAnnotation build() {
            if (inkPaths.isEmpty()) {
                throw new IllegalStateException("Ink annotation must have at least one path");
            }

            // Calculate bounding box from all paths
            double minX = Double.MAX_VALUE, minY = Double.MAX_VALUE;
            double maxX = Double.MIN_VALUE, maxY = Double.MIN_VALUE;

            for (List<Point> path : inkPaths) {
                for (Point p : path) {
                    minX = Math.min(minX, p.getX());
                    minY = Math.min(minY, p.getY());
                    maxX = Math.max(maxX, p.getX());
                    maxY = Math.max(maxY, p.getY());
                }
            }

            Rect rect = new Rect((float)minX, (float)minY, (float)(maxX - minX), (float)(maxY - minY));

            return new InkAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                new java.util.ArrayList<>(inkPaths),
                inkColor,
                inkThickness
            );
        }
    }

    @Override
    public String toString() {
        return String.format("InkAnnotation(%d strokes, %d points, rect=%s)",
            getStrokeCount(), getPointCount(), rect);
    }
}
