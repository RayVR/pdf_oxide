package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Text annotation (sticky note) for adding comments to PDF pages.
 *
 * <p>Text annotations appear as icons on the page that reveal their content
 * when clicked or hovered over. They're commonly used for reviewer comments.
 *
 * @since 1.0.0
 */
public final class TextAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final Optional<Color> color;
    private final Optional<String> iconName;
    private final Optional<String> replyTo;

    /**
     * Constructs a text annotation.
     *
     * @param rect location on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param color annotation color (optional)
     * @param iconName icon appearance ("Comment", "Note", "Help", "Key", "NewParagraph", "Paragraph", "Insert")
     * @param replyTo ID of annotation this replies to (optional)
     */
    private TextAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            Optional<Color> color,
            Optional<String> iconName,
            Optional<String> replyTo) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.color = color;
        this.iconName = iconName;
        this.replyTo = replyTo;
    }

    @Override
    public String getType() {
        return "Text";
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
     * Gets the icon appearance.
     *
     * @return icon name, default is "Note"
     */
    public String getIconName() {
        return iconName.orElse("Note");
    }

    /**
     * Gets the annotation color.
     *
     * @return color, empty if default
     */
    public Optional<Color> getColor() {
        return color;
    }

    /**
     * Gets the ID of the annotation this is a reply to.
     *
     * @return parent annotation ID, empty if not a reply
     */
    public Optional<String> getReplyTo() {
        return replyTo;
    }

    /**
     * Creates a builder for text annotations.
     *
     * @param rect location on page
     * @param contents annotation text
     * @return builder
     */
    public static Builder builder(Rect rect, String contents) {
        return new Builder(rect, contents);
    }

    /**
     * Fluent builder for text annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final String contents;
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Color> color = Optional.empty();
        private Optional<String> iconName = Optional.empty();
        private Optional<String> replyTo = Optional.empty();

        private Builder(Rect rect, String contents) {
            this.rect = rect;
            this.contents = contents;
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
         * @param color annotation color
         * @return this builder
         */
        public Builder color(Color color) {
            this.color = Optional.of(color);
            return this;
        }

        /**
         * Sets the icon appearance.
         *
         * @param iconName icon type ("Comment", "Note", "Help", "Key", "NewParagraph", "Paragraph", "Insert")
         * @return this builder
         */
        public Builder iconName(String iconName) {
            this.iconName = Optional.of(iconName);
            return this;
        }

        /**
         * Sets this as a reply to another annotation.
         *
         * @param parentId ID of parent annotation
         * @return this builder
         */
        public Builder replyTo(String parentId) {
            this.replyTo = Optional.of(parentId);
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return text annotation
         */
        public TextAnnotation build() {
            return new TextAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                color,
                iconName,
                replyTo
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "TextAnnotation(icon=%s, rect=%s, contents='%s')",
            getIconName(),
            rect,
            contents.length() > 30 ? contents.substring(0, 30) + "..." : contents
        );
    }
}
