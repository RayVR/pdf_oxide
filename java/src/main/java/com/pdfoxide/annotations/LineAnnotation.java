package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Point;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Line annotation for drawing lines with optional endpoints and captions.
 *
 * <p>Line annotations draw lines between two points with optional arrowheads,
 * captions, and styling options.
 *
 * @since 1.0.0
 */
public final class LineAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final Point startPoint;
    private final Point endPoint;
    private final Optional<Color> lineColor;
    private final double lineWidth;
    private final Optional<LineEndStyle> startEndStyle;
    private final Optional<LineEndStyle> endEndStyle;
    private final Optional<String> caption;
    private final Optional<Color> captionBackgroundColor;

    /**
     * Constructs a line annotation.
     *
     * @param rect bounding box
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param startPoint line start point
     * @param endPoint line end point
     * @param lineColor line color (optional)
     * @param lineWidth line width in points
     * @param startEndStyle start endpoint style (optional)
     * @param endEndStyle end endpoint style (optional)
     * @param caption line caption (optional)
     * @param captionBackgroundColor caption background (optional)
     */
    private LineAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            Point startPoint,
            Point endPoint,
            Optional<Color> lineColor,
            double lineWidth,
            Optional<LineEndStyle> startEndStyle,
            Optional<LineEndStyle> endEndStyle,
            Optional<String> caption,
            Optional<Color> captionBackgroundColor) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.startPoint = startPoint;
        this.endPoint = endPoint;
        this.lineColor = lineColor;
        this.lineWidth = lineWidth;
        this.startEndStyle = startEndStyle;
        this.endEndStyle = endEndStyle;
        this.caption = caption;
        this.captionBackgroundColor = captionBackgroundColor;
    }

    @Override
    public String getType() {
        return "Line";
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

    public Point getStartPoint() {
        return startPoint;
    }

    public Point getEndPoint() {
        return endPoint;
    }

    public Color getLineColor() {
        return lineColor.orElse(Color.BLACK);
    }

    public double getLineWidth() {
        return lineWidth;
    }

    public Optional<LineEndStyle> getStartEndStyle() {
        return startEndStyle;
    }

    public Optional<LineEndStyle> getEndEndStyle() {
        return endEndStyle;
    }

    public Optional<String> getCaption() {
        return caption;
    }

    public Optional<Color> getCaptionBackgroundColor() {
        return captionBackgroundColor;
    }

    public static Builder builder(Point startPoint, Point endPoint) {
        return new Builder(startPoint, endPoint);
    }

    public static final class Builder {
        private final Point startPoint;
        private final Point endPoint;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Color> lineColor = Optional.empty();
        private double lineWidth = 1.0;
        private Optional<LineEndStyle> startEndStyle = Optional.empty();
        private Optional<LineEndStyle> endEndStyle = Optional.empty();
        private Optional<String> caption = Optional.empty();
        private Optional<Color> captionBackgroundColor = Optional.empty();

        private Builder(Point startPoint, Point endPoint) {
            this.startPoint = startPoint;
            this.endPoint = endPoint;
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

        public Builder lineColor(Color color) {
            this.lineColor = Optional.of(color);
            return this;
        }

        public Builder lineWidth(double width) {
            this.lineWidth = width;
            return this;
        }

        public Builder startEndStyle(LineEndStyle style) {
            this.startEndStyle = Optional.of(style);
            return this;
        }

        public Builder endEndStyle(LineEndStyle style) {
            this.endEndStyle = Optional.of(style);
            return this;
        }

        public Builder caption(String caption) {
            this.caption = Optional.of(caption);
            return this;
        }

        public Builder captionBackgroundColor(Color color) {
            this.captionBackgroundColor = Optional.of(color);
            return this;
        }

        public LineAnnotation build() {
            Rect rect = new Rect(
                (float)Math.min(startPoint.getX(), endPoint.getX()),
                (float)Math.min(startPoint.getY(), endPoint.getY()),
                (float)(Math.abs(endPoint.getX() - startPoint.getX()) + lineWidth),
                (float)(Math.abs(endPoint.getY() - startPoint.getY()) + lineWidth)
            );

            return new LineAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                startPoint,
                endPoint,
                lineColor,
                lineWidth,
                startEndStyle,
                endEndStyle,
                caption,
                captionBackgroundColor
            );
        }
    }

    @Override
    public String toString() {
        return String.format("LineAnnotation(from %s to %s)", startPoint, endPoint);
    }
}

/**
 * Line endpoint styles for arrows and other line endings.
 */
final class LineEndStyle {
    public enum Type {
        NONE,           // No endpoint
        SQUARE,         // Square fill
        CIRCLE,         // Circle fill
        DIAMOND,        // Diamond fill
        OPEN_ARROW,     // Open arrow head
        CLOSED_ARROW,   // Closed/filled arrow head
        BUTT,           // Butt line end
        REVERSE_OPEN_ARROW,      // Reverse open arrow
        REVERSE_CLOSED_ARROW,    // Reverse closed arrow
        SLASH           // Slash line
    }

    private final Type type;
    private final boolean fill;

    private LineEndStyle(Type type, boolean fill) {
        this.type = type;
        this.fill = fill;
    }

    public static LineEndStyle of(Type type) {
        return new LineEndStyle(type, false);
    }

    public static LineEndStyle arrow() {
        return new LineEndStyle(Type.CLOSED_ARROW, true);
    }

    public static LineEndStyle openArrow() {
        return new LineEndStyle(Type.OPEN_ARROW, false);
    }

    public Type getType() {
        return type;
    }

    public boolean isFilled() {
        return fill;
    }

    @Override
    public String toString() {
        return String.format("%s%s", type, fill ? " (filled)" : "");
    }
}
