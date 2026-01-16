package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.net.URI;
import java.time.Instant;
import java.util.Optional;

/**
 * Link annotation for clickable URLs or internal page references.
 *
 * <p>Link annotations create clickable areas that navigate to external URLs,
 * email addresses, or other pages within the PDF.
 *
 * @since 1.0.0
 */
public final class LinkAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final LinkAction action;
    private final Optional<String> highlightMode;
    private final Optional<String> borderStyle;

    /**
     * Constructs a link annotation.
     *
     * @param rect clickable area
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param action link target action
     * @param highlightMode highlighting effect ("None", "Invert", "Outline", "Push")
     * @param borderStyle border appearance (optional)
     */
    private LinkAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            LinkAction action,
            Optional<String> highlightMode,
            Optional<String> borderStyle) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.action = action;
        this.highlightMode = highlightMode;
        this.borderStyle = borderStyle;
    }

    @Override
    public String getType() {
        return "Link";
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
     * Gets the link action/target.
     *
     * @return link action
     */
    public LinkAction getAction() {
        return action;
    }

    /**
     * Gets the highlight effect when clicked.
     *
     * @return highlight mode ("None", "Invert", "Outline", "Push")
     */
    public String getHighlightMode() {
        return highlightMode.orElse("Invert");
    }

    /**
     * Gets the border style.
     *
     * @return border appearance, empty for default
     */
    public Optional<String> getBorderStyle() {
        return borderStyle;
    }

    /**
     * Creates a builder for link annotations.
     *
     * @param rect clickable area
     * @param action link target
     * @return builder
     */
    public static Builder builder(Rect rect, LinkAction action) {
        return new Builder(rect, action);
    }

    /**
     * Fluent builder for link annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final LinkAction action;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> highlightMode = Optional.empty();
        private Optional<String> borderStyle = Optional.empty();

        private Builder(Rect rect, LinkAction action) {
            this.rect = rect;
            this.action = action;
        }

        /**
         * Sets the annotation content/label.
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
         * Sets the highlight effect.
         *
         * @param mode highlighting effect ("None", "Invert", "Outline", "Push")
         * @return this builder
         */
        public Builder highlightMode(String mode) {
            this.highlightMode = Optional.of(mode);
            return this;
        }

        /**
         * Sets the border style.
         *
         * @param style border appearance
         * @return this builder
         */
        public Builder borderStyle(String style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return link annotation
         */
        public LinkAnnotation build() {
            return new LinkAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                action,
                highlightMode,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format("LinkAnnotation(action=%s, rect=%s)", action, rect);
    }
}
