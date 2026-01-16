package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Screen annotation for embedded interactive screen content.
 *
 * <p>Screen annotations display interactive content that can render or play
 * media files referenced by a URI or embedded directly in the PDF.
 *
 * @since 1.0.0
 */
public final class ScreenAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String title;
    private final String resourceUri;
    private final String mimeType;
    private final Optional<String> thumbnailPath;
    private final boolean isRendered;

    /**
     * Constructs a screen annotation.
     *
     * @param rect location and size on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param title screen title/label
     * @param resourceUri URI or path to resource
     * @param mimeType MIME type of resource
     * @param thumbnailPath path to thumbnail image (optional)
     * @param isRendered true if resource has been rendered
     */
    private ScreenAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String title,
            String resourceUri,
            String mimeType,
            Optional<String> thumbnailPath,
            boolean isRendered) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.title = title;
        this.resourceUri = resourceUri;
        this.mimeType = mimeType;
        this.thumbnailPath = thumbnailPath;
        this.isRendered = isRendered;
    }

    @Override
    public String getType() {
        return "Screen";
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
     * Gets the screen title.
     *
     * @return title text
     */
    public String getTitle() {
        return title;
    }

    /**
     * Gets the resource URI or path.
     *
     * @return URI/path to resource
     */
    public String getResourceUri() {
        return resourceUri;
    }

    /**
     * Gets the MIME type.
     *
     * @return MIME type
     */
    public String getMimeType() {
        return mimeType;
    }

    /**
     * Gets the thumbnail path.
     *
     * @return path to thumbnail image, empty if none
     */
    public Optional<String> getThumbnailPath() {
        return thumbnailPath;
    }

    /**
     * Checks if resource has been rendered.
     *
     * @return true if rendered
     */
    public boolean isRendered() {
        return isRendered;
    }

    public static Builder builder(Rect rect, String title, String resourceUri, String mimeType) {
        return new Builder(rect, title, resourceUri, mimeType);
    }

    public static final class Builder {
        private final Rect rect;
        private final String title;
        private final String resourceUri;
        private final String mimeType;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> thumbnailPath = Optional.empty();
        private boolean isRendered = false;

        private Builder(Rect rect, String title, String resourceUri, String mimeType) {
            this.rect = rect;
            this.title = title;
            this.resourceUri = resourceUri;
            this.mimeType = mimeType;
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

        public Builder thumbnailPath(String path) {
            this.thumbnailPath = Optional.of(path);
            return this;
        }

        public Builder rendered(boolean rendered) {
            this.isRendered = rendered;
            return this;
        }

        public ScreenAnnotation build() {
            return new ScreenAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                title,
                resourceUri,
                mimeType,
                thumbnailPath,
                isRendered
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "ScreenAnnotation(title='%s', type=%s, rendered=%s, rect=%s)",
            title,
            mimeType,
            isRendered,
            rect
        );
    }
}
