package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Rich media annotation for embedding rich media content.
 *
 * <p>Rich media annotations embed rich media (Flash, HTML, etc.) that can be
 * rendered interactively within the PDF viewer.
 *
 * @since 1.0.0
 */
public final class RichMediaAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String title;
    private final RichMediaType mediaType;
    private final String resourceUri;
    private final Optional<String> activationMode;
    private final Optional<String> deactivationMode;

    /**
     * Rich media type.
     */
    public enum RichMediaType {
        FLASH,          // Flash animation
        HTML,           // HTML content
        JAVASCRIPT,     // JavaScript content
        PDF,            // Embedded PDF
        XHTML,          // XHTML content
        SVG,            // SVG graphics
        UNKNOWN         // Unknown type
    }

    /**
     * Constructs a rich media annotation.
     *
     * @param rect location and size on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param title media title
     * @param mediaType type of rich media
     * @param resourceUri URI or path to resource
     * @param activationMode activation mode (optional)
     * @param deactivationMode deactivation mode (optional)
     */
    private RichMediaAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String title,
            RichMediaType mediaType,
            String resourceUri,
            Optional<String> activationMode,
            Optional<String> deactivationMode) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.title = title;
        this.mediaType = mediaType;
        this.resourceUri = resourceUri;
        this.activationMode = activationMode;
        this.deactivationMode = deactivationMode;
    }

    @Override
    public String getType() {
        return "RichMedia";
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

    public String getTitle() {
        return title;
    }

    public RichMediaType getMediaType() {
        return mediaType;
    }

    public String getResourceUri() {
        return resourceUri;
    }

    public Optional<String> getActivationMode() {
        return activationMode;
    }

    public Optional<String> getDeactivationMode() {
        return deactivationMode;
    }

    public static Builder builder(Rect rect, String title, RichMediaType type, String resourceUri) {
        return new Builder(rect, title, type, resourceUri);
    }

    public static final class Builder {
        private final Rect rect;
        private final String title;
        private final RichMediaType mediaType;
        private final String resourceUri;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> activationMode = Optional.empty();
        private Optional<String> deactivationMode = Optional.empty();

        private Builder(Rect rect, String title, RichMediaType type, String resourceUri) {
            this.rect = rect;
            this.title = title;
            this.mediaType = type;
            this.resourceUri = resourceUri;
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

        public Builder activationMode(String mode) {
            this.activationMode = Optional.of(mode);
            return this;
        }

        public Builder deactivationMode(String mode) {
            this.deactivationMode = Optional.of(mode);
            return this;
        }

        public RichMediaAnnotation build() {
            return new RichMediaAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                title,
                mediaType,
                resourceUri,
                activationMode,
                deactivationMode
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "RichMediaAnnotation(title='%s', type=%s, rect=%s)",
            title,
            mediaType,
            rect
        );
    }
}
