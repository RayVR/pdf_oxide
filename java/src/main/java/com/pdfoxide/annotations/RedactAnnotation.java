package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Redaction annotation for masking sensitive content.
 *
 * <p>Redaction annotations mark regions for redaction (permanent removal or masking).
 * When applied, they can permanently remove or blank out the underlying content.
 *
 * @since 1.0.0
 */
public final class RedactAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final List<Rect> quadPoints;
    private final Optional<Color> fillColor;
    private final Optional<Color> outlineColor;
    private final Optional<String> overlayText;
    private final boolean isApplied;

    /**
     * Constructs a redaction annotation.
     *
     * @param rect bounding box
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param quadPoints regions to redact
     * @param fillColor redaction fill color (optional)
     * @param outlineColor border color (optional)
     * @param overlayText text to display over redaction (optional)
     * @param isApplied true if redaction has been applied
     */
    private RedactAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            List<Rect> quadPoints,
            Optional<Color> fillColor,
            Optional<Color> outlineColor,
            Optional<String> overlayText,
            boolean isApplied) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.quadPoints = quadPoints;
        this.fillColor = fillColor;
        this.outlineColor = outlineColor;
        this.overlayText = overlayText;
        this.isApplied = isApplied;
    }

    @Override
    public String getType() {
        return "Redact";
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
     * Gets the regions to redact.
     *
     * @return list of rectangles (immutable)
     */
    public List<Rect> getQuadPoints() {
        return Collections.unmodifiableList(quadPoints);
    }

    /**
     * Gets the fill color.
     *
     * @return redaction color, default is black
     */
    public Color getFillColor() {
        return fillColor.orElse(Color.BLACK);
    }

    /**
     * Gets the outline color.
     *
     * @return border color, empty for no border
     */
    public Optional<Color> getOutlineColor() {
        return outlineColor;
    }

    /**
     * Gets overlay text to display over redaction.
     *
     * @return text, empty for blank redaction
     */
    public Optional<String> getOverlayText() {
        return overlayText;
    }

    /**
     * Checks if redaction has been applied.
     *
     * @return true if content has been permanently redacted
     */
    public boolean isApplied() {
        return isApplied;
    }

    /**
     * Creates a builder for redaction annotations.
     *
     * @param rect region to redact
     * @return builder
     */
    public static Builder builder(Rect rect) {
        return new Builder(rect);
    }

    /**
     * Fluent builder for redaction annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private List<Rect> quadPoints = Collections.emptyList();
        private Optional<Color> fillColor = Optional.empty();
        private Optional<Color> outlineColor = Optional.empty();
        private Optional<String> overlayText = Optional.empty();
        private boolean isApplied = false;

        private Builder(Rect rect) {
            this.rect = rect;
            this.quadPoints = Collections.singletonList(rect);
        }

        /**
         * Sets the content/reason for redaction.
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
         * Sets the regions to redact.
         *
         * @param quadPoints list of rectangles
         * @return this builder
         */
        public Builder quadPoints(List<Rect> quadPoints) {
            this.quadPoints = quadPoints;
            return this;
        }

        /**
         * Sets the fill color.
         *
         * @param color redaction color
         * @return this builder
         */
        public Builder fillColor(Color color) {
            this.fillColor = Optional.of(color);
            return this;
        }

        /**
         * Sets the outline color.
         *
         * @param color border color
         * @return this builder
         */
        public Builder outlineColor(Color color) {
            this.outlineColor = Optional.of(color);
            return this;
        }

        /**
         * Sets overlay text to display over redaction.
         *
         * @param text overlay text
         * @return this builder
         */
        public Builder overlayText(String text) {
            this.overlayText = Optional.of(text);
            return this;
        }

        /**
         * Marks as applied (permanently redacted).
         *
         * @param applied true if redaction is applied
         * @return this builder
         */
        public Builder applied(boolean applied) {
            this.isApplied = applied;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return redaction annotation
         */
        public RedactAnnotation build() {
            return new RedactAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                quadPoints,
                fillColor,
                outlineColor,
                overlayText,
                isApplied
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "RedactAnnotation(status=%s, regions=%d, rect=%s)",
            isApplied ? "APPLIED" : "PENDING",
            quadPoints.size(),
            rect
        );
    }
}
