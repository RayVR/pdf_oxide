package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Stamp annotation for predefined appearance stamps.
 *
 * <p>Stamp annotations display standard predefined stamps like "APPROVED",
 * "DRAFT", "CONFIDENTIAL", etc. They're commonly used for document status marking.
 *
 * @since 1.0.0
 */
public final class StampAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final StampType stampType;
    private final Optional<String> customAppearance;

    /**
     * Constructs a stamp annotation.
     *
     * @param rect location on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param stampType stamp appearance
     * @param customAppearance custom appearance (optional, overrides stampType)
     */
    private StampAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            StampType stampType,
            Optional<String> customAppearance) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.stampType = stampType;
        this.customAppearance = customAppearance;
    }

    @Override
    public String getType() {
        return "Stamp";
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
     * Gets the stamp type.
     *
     * @return standard stamp type
     */
    public StampType getStampType() {
        return stampType;
    }

    /**
     * Gets custom appearance if set.
     *
     * @return custom appearance text, empty if using standard stamp
     */
    public Optional<String> getCustomAppearance() {
        return customAppearance;
    }

    /**
     * Gets the display text (standard or custom).
     *
     * @return stamp text
     */
    public String getDisplayText() {
        return customAppearance.orElse(stampType.toString().replace("_", " "));
    }

    /**
     * Creates a builder for stamp annotations.
     *
     * @param rect location on page
     * @param stampType stamp appearance
     * @return builder
     */
    public static Builder builder(Rect rect, StampType stampType) {
        return new Builder(rect, stampType);
    }

    /**
     * Fluent builder for stamp annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final StampType stampType;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> customAppearance = Optional.empty();

        private Builder(Rect rect, StampType stampType) {
            this.rect = rect;
            this.stampType = stampType;
        }

        /**
         * Sets the annotation content.
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
         * Sets a custom appearance (overrides standard stamp type).
         *
         * @param appearance custom display text
         * @return this builder
         */
        public Builder customAppearance(String appearance) {
            this.customAppearance = Optional.of(appearance);
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return stamp annotation
         */
        public StampAnnotation build() {
            return new StampAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                stampType,
                customAppearance
            );
        }
    }

    @Override
    public String toString() {
        return String.format("StampAnnotation(stamp=%s, rect=%s)", getDisplayText(), rect);
    }
}
