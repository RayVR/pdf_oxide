package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Free text annotation for adding text content directly visible on the page.
 *
 * <p>Free text annotations display text content directly on the page surface,
 * unlike text annotations (sticky notes) which appear as icons. They're useful
 * for adding comments, labels, or instructions visible without clicking.
 *
 * @since 1.0.0
 */
public final class FreeTextAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String fontName;
    private final double fontSize;
    private final Optional<Color> textColor;
    private final Optional<Color> backgroundColor;
    private final Optional<Color> borderColor;
    private final double borderWidth;
    private final int textAlignment;

    /**
     * Constructs a free text annotation.
     *
     * @param rect location and size on page
     * @param contents text to display
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param fontName font name (e.g., "Helvetica", "Times-Roman")
     * @param fontSize font size in points
     * @param textColor text color (optional)
     * @param backgroundColor background color (optional)
     * @param borderColor border color (optional)
     * @param borderWidth border width in points (0 = no border)
     * @param textAlignment text alignment (0=left, 1=center, 2=right)
     */
    private FreeTextAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String fontName,
            double fontSize,
            Optional<Color> textColor,
            Optional<Color> backgroundColor,
            Optional<Color> borderColor,
            double borderWidth,
            int textAlignment) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.fontName = fontName;
        this.fontSize = fontSize;
        this.textColor = textColor;
        this.backgroundColor = backgroundColor;
        this.borderColor = borderColor;
        this.borderWidth = borderWidth;
        this.textAlignment = textAlignment;
    }

    @Override
    public String getType() {
        return "FreeText";
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
     * Gets the font name.
     *
     * @return font name (e.g., "Helvetica")
     */
    public String getFontName() {
        return fontName;
    }

    /**
     * Gets the font size.
     *
     * @return font size in points
     */
    public double getFontSize() {
        return fontSize;
    }

    /**
     * Gets the text color.
     *
     * @return text color, default is black
     */
    public Color getTextColor() {
        return textColor.orElse(Color.BLACK);
    }

    /**
     * Gets the background color.
     *
     * @return background color, empty if transparent
     */
    public Optional<Color> getBackgroundColor() {
        return backgroundColor;
    }

    /**
     * Gets the border color.
     *
     * @return border color, default is black
     */
    public Color getBorderColor() {
        return borderColor.orElse(Color.BLACK);
    }

    /**
     * Gets the border width.
     *
     * @return border width in points (0 = no border)
     */
    public double getBorderWidth() {
        return borderWidth;
    }

    /**
     * Gets the text alignment.
     *
     * @return 0=left, 1=center, 2=right
     */
    public int getTextAlignment() {
        return textAlignment;
    }

    /**
     * Gets the text alignment as a string.
     *
     * @return "LEFT", "CENTER", or "RIGHT"
     */
    public String getTextAlignmentName() {
        switch (textAlignment) {
            case 0: return "LEFT";
            case 1: return "CENTER";
            case 2: return "RIGHT";
            default: return "LEFT";
        }
    }

    /**
     * Creates a builder for free text annotations.
     *
     * @param rect location and size
     * @param contents text to display
     * @return builder
     */
    public static Builder builder(Rect rect, String contents) {
        return new Builder(rect, contents);
    }

    /**
     * Fluent builder for free text annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final String contents;
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private String fontName = "Helvetica";
        private double fontSize = 12.0;
        private Optional<Color> textColor = Optional.empty();
        private Optional<Color> backgroundColor = Optional.empty();
        private Optional<Color> borderColor = Optional.empty();
        private double borderWidth = 1.0;
        private int textAlignment = 0;  // LEFT

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
         * Sets the font.
         *
         * @param fontName font name
         * @param fontSize font size in points
         * @return this builder
         */
        public Builder font(String fontName, double fontSize) {
            this.fontName = fontName;
            this.fontSize = fontSize;
            return this;
        }

        /**
         * Sets the text color.
         *
         * @param color text color
         * @return this builder
         */
        public Builder textColor(Color color) {
            this.textColor = Optional.of(color);
            return this;
        }

        /**
         * Sets the background color.
         *
         * @param color background color
         * @return this builder
         */
        public Builder backgroundColor(Color color) {
            this.backgroundColor = Optional.of(color);
            return this;
        }

        /**
         * Sets the border.
         *
         * @param color border color
         * @param width border width in points
         * @return this builder
         */
        public Builder border(Color color, double width) {
            this.borderColor = Optional.of(color);
            this.borderWidth = width;
            return this;
        }

        /**
         * Sets the text alignment.
         *
         * @param alignment 0=left, 1=center, 2=right
         * @return this builder
         */
        public Builder textAlignment(int alignment) {
            this.textAlignment = Math.min(2, Math.max(0, alignment));
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return free text annotation
         */
        public FreeTextAnnotation build() {
            return new FreeTextAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                fontName,
                fontSize,
                textColor,
                backgroundColor,
                borderColor,
                borderWidth,
                textAlignment
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "FreeTextAnnotation(font=%s %.1fpt, align=%s, rect=%s)",
            fontName,
            fontSize,
            getTextAlignmentName(),
            rect
        );
    }
}
