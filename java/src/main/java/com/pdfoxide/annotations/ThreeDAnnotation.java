package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * 3D annotation for embedding interactive 3D models in PDFs.
 *
 * <p>3D annotations embed 3D models (typically in U3D or PRC format) that
 * users can interact with, rotate, zoom, and manipulate within the PDF viewer.
 *
 * @since 1.0.0
 */
public final class ThreeDAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String title;
    private final String modelFilePath;
    private final ModelFormat format;
    private final Optional<String> thumbnailPath;
    private final boolean isEmbedded;
    private final double defaultZoom;
    private final boolean isInteractive;

    /**
     * 3D model format.
     */
    public enum ModelFormat {
        U3D,            // Universal 3D format
        PRC,            // Product Representation Compact
        OBJ,            // Wavefront OBJ
        DAE,            // COLLADA format
        UNKNOWN         // Unknown format
    }

    /**
     * Constructs a 3D annotation.
     *
     * @param rect location and size on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param title model title
     * @param modelFilePath path to 3D model file
     * @param format model format
     * @param thumbnailPath path to thumbnail image (optional)
     * @param isEmbedded true if model is embedded in PDF
     * @param defaultZoom default zoom level
     * @param isInteractive true if user can interact with model
     */
    private ThreeDAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String title,
            String modelFilePath,
            ModelFormat format,
            Optional<String> thumbnailPath,
            boolean isEmbedded,
            double defaultZoom,
            boolean isInteractive) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.title = title;
        this.modelFilePath = modelFilePath;
        this.format = format;
        this.thumbnailPath = thumbnailPath;
        this.isEmbedded = isEmbedded;
        this.defaultZoom = defaultZoom;
        this.isInteractive = isInteractive;
    }

    @Override
    public String getType() {
        return "3D";
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

    public String getModelFilePath() {
        return modelFilePath;
    }

    public ModelFormat getFormat() {
        return format;
    }

    public Optional<String> getThumbnailPath() {
        return thumbnailPath;
    }

    public boolean isEmbedded() {
        return isEmbedded;
    }

    public double getDefaultZoom() {
        return defaultZoom;
    }

    public boolean isInteractive() {
        return isInteractive;
    }

    public static Builder builder(Rect rect, String title, String modelPath, ModelFormat format) {
        return new Builder(rect, title, modelPath, format);
    }

    public static final class Builder {
        private final Rect rect;
        private final String title;
        private final String modelFilePath;
        private final ModelFormat format;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> thumbnailPath = Optional.empty();
        private boolean isEmbedded = false;
        private double defaultZoom = 1.0;
        private boolean isInteractive = true;

        private Builder(Rect rect, String title, String modelPath, ModelFormat format) {
            this.rect = rect;
            this.title = title;
            this.modelFilePath = modelPath;
            this.format = format;
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

        public Builder embedded(boolean embedded) {
            this.isEmbedded = embedded;
            return this;
        }

        public Builder defaultZoom(double zoom) {
            this.defaultZoom = Math.max(0.1, zoom);
            return this;
        }

        public Builder interactive(boolean interactive) {
            this.isInteractive = interactive;
            return this;
        }

        public ThreeDAnnotation build() {
            return new ThreeDAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                title,
                modelFilePath,
                format,
                thumbnailPath,
                isEmbedded,
                defaultZoom,
                isInteractive
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "ThreeDAnnotation(title='%s', format=%s, interactive=%s, rect=%s)",
            title,
            format,
            isInteractive,
            rect
        );
    }
}
