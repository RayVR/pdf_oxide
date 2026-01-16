package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Point;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Text markup annotation for highlighting, underlining, striking out, or squiggly underlining text.
 *
 * <p>Markup annotations mark up text in the document with visual indicators
 * like highlights, underlines, or strikethrough effects.
 *
 * @since 1.0.0
 */
public final class HighlightAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final HighlightMode mode;
    private final Optional<Color> color;
    private final List<Rect> quadPoints;

    /**
     * Constructs a highlight annotation.
     *
     * @param rect bounding box
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param mode highlight mode (HIGHLIGHT, UNDERLINE, STRIKEOUT, SQUIGGLY)
     * @param color annotation color (optional)
     * @param quadPoints precise text areas (4 corners per selected text area)
     */
    private HighlightAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            HighlightMode mode,
            Optional<Color> color,
            List<Rect> quadPoints) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.mode = mode;
        this.color = color;
        this.quadPoints = quadPoints;
    }

    @Override
    public String getType() {
        return mode.name();  // "HIGHLIGHT", "UNDERLINE", etc.
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
     * Gets the highlight mode.
     *
     * @return highlight type
     */
    public HighlightMode getMode() {
        return mode;
    }

    /**
     * Gets the color.
     *
     * @return highlight color, default is yellow
     */
    public Color getColor() {
        return color.orElse(Color.YELLOW);
    }

    /**
     * Gets the precise text areas marked for highlighting.
     *
     * <p>Each Rect represents the bounding box of a selected text region.
     * Multiple rectangles can represent non-contiguous selections.
     *
     * @return list of quad rectangles (immutable)
     */
    public List<Rect> getQuadPoints() {
        return Collections.unmodifiableList(quadPoints);
    }

    /**
     * Creates a builder for highlight annotations.
     *
     * @param rect bounding box
     * @param mode highlight mode
     * @return builder
     */
    public static Builder builder(Rect rect, HighlightMode mode) {
        return new Builder(rect, mode);
    }

    /**
     * Fluent builder for highlight annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final HighlightMode mode;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Color> color = Optional.empty();
        private List<Rect> quadPoints = Collections.emptyList();

        private Builder(Rect rect, HighlightMode mode) {
            this.rect = rect;
            this.mode = mode;
        }

        /**
         * Sets the annotation content/comment.
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
         * Sets the color.
         *
         * @param color highlight color
         * @return this builder
         */
        public Builder color(Color color) {
            this.color = Optional.of(color);
            return this;
        }

        /**
         * Sets the precise text areas to highlight.
         *
         * @param quadPoints list of rectangles representing selected text
         * @return this builder
         */
        public Builder quadPoints(List<Rect> quadPoints) {
            this.quadPoints = quadPoints;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return highlight annotation
         */
        public HighlightAnnotation build() {
            return new HighlightAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                mode,
                color,
                quadPoints
            );
        }
    }

    @Override
    public String toString() {
        return String.format("HighlightAnnotation(mode=%s, rect=%s, color=%s)", mode, rect, getColor());
    }
}
