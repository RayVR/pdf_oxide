package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Popup annotation for displaying annotation windows.
 *
 * <p>Popup annotations display content in a floating window that can be
 * opened or closed by the user. They're often associated with other
 * annotations like text or markup annotations.
 *
 * @since 1.0.0
 */
public final class PopupAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final Optional<String> parentAnnotationId;
    private final boolean isOpen;

    /**
     * Constructs a popup annotation.
     *
     * @param rect location and size on page
     * @param contents popup content
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param parentAnnotationId ID of parent annotation (optional)
     * @param isOpen true if popup is initially open
     */
    private PopupAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            Optional<String> parentAnnotationId,
            boolean isOpen) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.parentAnnotationId = parentAnnotationId;
        this.isOpen = isOpen;
    }

    @Override
    public String getType() {
        return "Popup";
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
     * Gets the parent annotation ID.
     *
     * @return parent annotation reference, empty if standalone
     */
    public Optional<String> getParentAnnotationId() {
        return parentAnnotationId;
    }

    /**
     * Checks if popup is initially open.
     *
     * @return true if open by default
     */
    public boolean isOpen() {
        return isOpen;
    }

    /**
     * Creates a builder for popup annotations.
     *
     * @param rect location and size
     * @param contents popup content
     * @return builder
     */
    public static Builder builder(Rect rect, String contents) {
        return new Builder(rect, contents);
    }

    /**
     * Fluent builder for popup annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final String contents;
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> parentAnnotationId = Optional.empty();
        private boolean isOpen = false;

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
         * Sets the parent annotation.
         *
         * @param parentId parent annotation ID
         * @return this builder
         */
        public Builder parentAnnotation(String parentId) {
            this.parentAnnotationId = Optional.of(parentId);
            return this;
        }

        /**
         * Sets if popup is initially open.
         *
         * @param open true for open, false for closed
         * @return this builder
         */
        public Builder open(boolean open) {
            this.isOpen = open;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return popup annotation
         */
        public PopupAnnotation build() {
            return new PopupAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                parentAnnotationId,
                isOpen
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "PopupAnnotation(status=%s, rect=%s)",
            isOpen ? "OPEN" : "CLOSED",
            rect
        );
    }
}
