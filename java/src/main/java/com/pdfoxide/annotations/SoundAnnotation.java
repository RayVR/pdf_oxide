package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Optional;

/**
 * Sound annotation for embedding audio clips in PDFs.
 *
 * <p>Sound annotations embed audio files that users can play by clicking.
 * Useful for adding voice notes, instructions, or narration to documents.
 *
 * @since 1.0.0
 */
public final class SoundAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String soundFilePath;
    private final String soundFileName;
    private final Optional<String> encoding;
    private final Optional<Double> sampleRate;
    private final Optional<Integer> channels;
    private final SoundIcon icon;

    /**
     * Sound annotation icon types.
     */
    public enum SoundIcon {
        SPEAKER,        // Speaker icon
        MICROPHONE      // Microphone icon
    }

    /**
     * Constructs a sound annotation.
     *
     * @param rect location on page
     * @param contents annotation text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param soundFilePath path to audio file
     * @param soundFileName original file name
     * @param encoding audio encoding (optional, e.g., "PCM", "MP3")
     * @param sampleRate sample rate in Hz (optional)
     * @param channels number of channels (optional)
     * @param icon icon appearance
     */
    private SoundAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String soundFilePath,
            String soundFileName,
            Optional<String> encoding,
            Optional<Double> sampleRate,
            Optional<Integer> channels,
            SoundIcon icon) {
        this.rect = rect;
        this.contents = contents;
        this.author = author;
        this.createdDate = createdDate;
        this.modifiedDate = modifiedDate;
        this.subject = subject;
        this.flags = flags;
        this.soundFilePath = soundFilePath;
        this.soundFileName = soundFileName;
        this.encoding = encoding;
        this.sampleRate = sampleRate;
        this.channels = channels;
        this.icon = icon;
    }

    @Override
    public String getType() {
        return "Sound";
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

    public String getSoundFilePath() {
        return soundFilePath;
    }

    public String getSoundFileName() {
        return soundFileName;
    }

    public Optional<String> getEncoding() {
        return encoding;
    }

    public Optional<Double> getSampleRate() {
        return sampleRate;
    }

    public Optional<Integer> getChannels() {
        return channels;
    }

    public SoundIcon getIcon() {
        return icon;
    }

    public static Builder builder(Rect rect, Path soundPath) {
        return new Builder(rect, soundPath.toString());
    }

    public static Builder builder(Rect rect, String soundPath) {
        return new Builder(rect, soundPath);
    }

    public static final class Builder {
        private final Rect rect;
        private final String soundFilePath;
        private String contents = "";
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private Optional<String> encoding = Optional.empty();
        private Optional<Double> sampleRate = Optional.empty();
        private Optional<Integer> channels = Optional.empty();
        private SoundIcon icon = SoundIcon.SPEAKER;

        private Builder(Rect rect, String soundFilePath) {
            this.rect = rect;
            this.soundFilePath = soundFilePath;
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

        public Builder encoding(String encoding) {
            this.encoding = Optional.of(encoding);
            return this;
        }

        public Builder sampleRate(double rate) {
            this.sampleRate = Optional.of(rate);
            return this;
        }

        public Builder channels(int numChannels) {
            this.channels = Optional.of(numChannels);
            return this;
        }

        public Builder icon(SoundIcon icon) {
            this.icon = icon;
            return this;
        }

        public SoundAnnotation build() {
            String fileName = soundFilePath.substring(soundFilePath.lastIndexOf('/') + 1);
            if (fileName.isEmpty()) {
                fileName = soundFilePath.substring(soundFilePath.lastIndexOf('\\') + 1);
            }

            return new SoundAnnotation(
                rect,
                contents,
                author,
                createdDate,
                modifiedDate,
                subject,
                flags,
                soundFilePath,
                fileName,
                encoding,
                sampleRate,
                channels,
                icon
            );
        }
    }

    @Override
    public String toString() {
        return String.format("SoundAnnotation(file='%s', icon=%s, rect=%s)", soundFileName, icon, rect);
    }
}
