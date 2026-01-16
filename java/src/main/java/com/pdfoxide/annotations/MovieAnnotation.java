package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Optional;

/**
 * Movie annotation for embedding video clips in PDFs.
 *
 * <p>Movie annotations embed video files that users can play by clicking.
 * Useful for adding demonstrations, tutorials, or video content to documents.
 *
 * @since 1.0.0
 */
public final class MovieAnnotation extends Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String movieFilePath;
    private final String movieFileName;
    private final String mimeType;
    private final Optional<Integer> width;
    private final Optional<Integer> height;
    private final Optional<Double> duration;
    private final boolean autoPlay;
    private final boolean loop;

    /**
     * Constructs a movie annotation.
     *
     * @param rect location and size on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param movieFilePath path to video file
     * @param movieFileName original file name
     * @param mimeType MIME type (e.g., "video/mp4", "video/quicktime")
     * @param width video width in pixels (optional)
     * @param height video height in pixels (optional)
     * @param duration duration in seconds (optional)
     * @param autoPlay true to autoplay on opening
     * @param loop true to loop playback
     */
    private MovieAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String movieFilePath,
            String movieFileName,
            String mimeType,
            Optional<Integer> width,
            Optional<Integer> height,
            Optional<Double> duration,
            boolean autoPlay,
            boolean loop) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.movieFilePath = movieFilePath;
        this.movieFileName = movieFileName;
        this.mimeType = mimeType;
        this.width = width;
        this.height = height;
        this.duration = duration;
        this.autoPlay = autoPlay;
        this.loop = loop;
    }

    @Override
    public String getType() {
        return "Movie";
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

    public String getMovieFilePath() {
        return movieFilePath;
    }

    public String getMovieFileName() {
        return movieFileName;
    }

    public String getMimeType() {
        return mimeType;
    }

    public Optional<Integer> getWidth() {
        return width;
    }

    public Optional<Integer> getHeight() {
        return height;
    }

    public Optional<Double> getDuration() {
        return duration;
    }

    public boolean isAutoPlay() {
        return autoPlay;
    }

    public boolean isLoop() {
        return loop;
    }

    public static Builder builder(Rect rect, Path moviePath, String mimeType) {
        return new Builder(rect, moviePath.toString(), mimeType);
    }

    public static Builder builder(Rect rect, String moviePath, String mimeType) {
        return new Builder(rect, moviePath, mimeType);
    }

    public static final class Builder {
        private final Rect rect;
        private final String movieFilePath;
        private final String mimeType;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<Integer> width = Optional.empty();
        private Optional<Integer> height = Optional.empty();
        private Optional<Double> duration = Optional.empty();
        private boolean autoPlay = false;
        private boolean loop = false;

        private Builder(Rect rect, String movieFilePath, String mimeType) {
            this.rect = rect;
            this.movieFilePath = movieFilePath;
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

        public Builder dimensions(int width, int height) {
            this.width = Optional.of(width);
            this.height = Optional.of(height);
            return this;
        }

        public Builder duration(double seconds) {
            this.duration = Optional.of(seconds);
            return this;
        }

        public Builder autoPlay(boolean autoPlay) {
            this.autoPlay = autoPlay;
            return this;
        }

        public Builder loop(boolean loop) {
            this.loop = loop;
            return this;
        }

        public MovieAnnotation build() {
            String fileName = movieFilePath.substring(movieFilePath.lastIndexOf('/') + 1);
            if (fileName.isEmpty()) {
                fileName = movieFilePath.substring(movieFilePath.lastIndexOf('\\') + 1);
            }

            return new MovieAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                movieFilePath,
                fileName,
                mimeType,
                width,
                height,
                duration,
                autoPlay,
                loop
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "MovieAnnotation(file='%s', %s, auto=%s, rect=%s)",
            movieFileName,
            mimeType,
            autoPlay,
            rect
        );
    }
}
