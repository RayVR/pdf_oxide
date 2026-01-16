package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Optional;

/**
 * File attachment annotation for embedding files in PDFs.
 *
 * <p>File attachment annotations allow attaching files to a PDF, like email attachments.
 * Users can click the annotation to extract and open the attached file.
 *
 * @since 1.0.0
 */
public final class FileAttachmentAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String filePath;
    private final String fileName;
    private final Optional<String> description;
    private final FileAttachmentIcon icon;

    /**
     * File attachment icon types.
     */
    public enum FileAttachmentIcon {
        GRAPH,          // Graph icon
        PAPERCLIP,      // Paperclip icon
        PUSH_PIN,       // Push pin icon
        TAG             // Tag icon
    }

    /**
     * Constructs a file attachment annotation.
     *
     * @param rect location on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param filePath path to file to attach
     * @param fileName original file name
     * @param description file description (optional)
     * @param icon icon appearance
     */
    private FileAttachmentAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String filePath,
            String fileName,
            Optional<String> description,
            FileAttachmentIcon icon) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.filePath = filePath;
        this.fileName = fileName;
        this.description = description;
        this.icon = icon;
    }

    @Override
    public String getType() {
        return "FileAttachment";
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
     * Gets the file path.
     *
     * @return path to attached file
     */
    public String getFilePath() {
        return filePath;
    }

    /**
     * Gets the original file name.
     *
     * @return file name
     */
    public String getFileName() {
        return fileName;
    }

    /**
     * Gets the file description.
     *
     * @return description, empty if not set
     */
    public Optional<String> getDescription() {
        return description;
    }

    /**
     * Gets the icon type.
     *
     * @return icon appearance
     */
    public FileAttachmentIcon getIcon() {
        return icon;
    }

    /**
     * Creates a builder for file attachment annotations.
     *
     * @param rect location on page
     * @param filePath path to file to attach
     * @return builder
     */
    public static Builder builder(Rect rect, Path filePath) {
        return new Builder(rect, filePath.toString());
    }

    /**
     * Creates a builder for file attachment annotations.
     *
     * @param rect location on page
     * @param filePath path to file to attach
     * @return builder
     */
    public static Builder builder(Rect rect, String filePath) {
        return new Builder(rect, filePath);
    }

    /**
     * Fluent builder for file attachment annotations.
     */
    public static final class Builder {
        private final Rect rect;
        private final String filePath;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> description = Optional.empty();
        private FileAttachmentIcon icon = FileAttachmentIcon.PAPERCLIP;

        private Builder(Rect rect, String filePath) {
            this.rect = rect;
            this.filePath = filePath;
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
         * Sets the file description.
         *
         * @param description description text
         * @return this builder
         */
        public Builder description(String description) {
            this.description = Optional.of(description);
            return this;
        }

        /**
         * Sets the icon.
         *
         * @param icon icon type
         * @return this builder
         */
        public Builder icon(FileAttachmentIcon icon) {
            this.icon = icon;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return file attachment annotation
         */
        public FileAttachmentAnnotation build() {
            // Extract file name from path if not provided
            String fileName = filePath.substring(filePath.lastIndexOf('/') + 1);
            if (fileName.isEmpty()) {
                fileName = filePath.substring(filePath.lastIndexOf('\\') + 1);
            }

            return new FileAttachmentAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                filePath,
                fileName,
                description,
                icon
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "FileAttachmentAnnotation(file='%s', icon=%s, rect=%s)",
            fileName,
            icon,
            rect
        );
    }
}
