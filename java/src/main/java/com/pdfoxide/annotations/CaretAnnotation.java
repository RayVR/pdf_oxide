package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Caret annotation for indicating insertion points.
 *
 * <p>Caret annotations mark where text should be inserted or indicate
 * an editing position, similar to a cursor in a text editor.
 *
 * @since 1.0.0
 */
public final class CaretAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final CaretSymbol symbol;

    /**
     * Caret symbol types.
     */
    public enum CaretSymbol {
        DEFAULT,        // Default caret symbol
        PARAGRAPH       // Paragraph insertion point
    }

    /**
     * Constructs a caret annotation.
     *
     * @param rect location on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param symbol caret symbol type
     */
    private CaretAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            CaretSymbol symbol) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.symbol = symbol;
    }

    @Override
    public String getType() {
        return "Caret";
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
     * Gets the caret symbol.
     *
     * @return symbol type
     */
    public CaretSymbol getSymbol() {
        return symbol;
    }

    /**
     * Creates a builder for caret annotations.
     *
     * @param rect insertion point location
     * @return builder
     */
    public static Builder builder(Rect rect) {
        return new Builder(rect);
    }

    /**
     * Fluent builder for caret annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private CaretSymbol symbol = CaretSymbol.DEFAULT;

        private Builder(Rect rect) {
            this.rect = rect;
        }

        /**
         * Sets the content.
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
         * Sets the caret symbol.
         *
         * @param symbol symbol type
         * @return this builder
         */
        public Builder symbol(CaretSymbol symbol) {
            this.symbol = symbol;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return caret annotation
         */
        public CaretAnnotation build() {
            return new CaretAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                symbol
            );
        }
    }

    @Override
    public String toString() {
        return String.format("CaretAnnotation(symbol=%s, rect=%s)", symbol, rect);
    }
}
