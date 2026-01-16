package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Color;
import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Watermark annotation for adding watermarks to PDF pages.
 *
 * <p>Watermarks display text (usually translucent) across the page background,
 * commonly used for "DRAFT", "CONFIDENTIAL", "DO NOT COPY" indicators.
 *
 * @since 1.0.0
 */
public final class WatermarkAnnotation implements Annotation {
    private final Rect rect;
    private final String contents;
    private final Optional<String> author;
    private final Optional<Instant> createdDate;
    private final Optional<Instant> modifiedDate;
    private final Optional<String> subject;
    private final int flags;
    private final String fontName;
    private final double fontSize;
    private final Color textColor;
    private final double opacity;
    private final double rotationDegrees;
    private final boolean fixedPosition;

    /**
     * Constructs a watermark annotation.
     *
     * @param rect location on page
     * @param contents watermark text
     * @param author creator name (optional)
     * @param createdDate creation timestamp (optional)
     * @param modifiedDate modification timestamp (optional)
     * @param subject annotation subject (optional)
     * @param flags display flags
     * @param fontName font name
     * @param fontSize font size in points
     * @param textColor text color
     * @param opacity opacity (0.0-1.0, where 0.5 is 50% transparent)
     * @param rotationDegrees rotation angle in degrees
     * @param fixedPosition true if watermark position is fixed relative to page
     */
    private WatermarkAnnotation(
            Rect rect,
            String contents,
            Optional<String> author,
            Optional<Instant> createdDate,
            Optional<Instant> modifiedDate,
            Optional<String> subject,
            int flags,
            String fontName,
            double fontSize,
            Color textColor,
            double opacity,
            double rotationDegrees,
            boolean fixedPosition) {
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
        this.opacity = Math.min(1.0, Math.max(0.0, opacity));
        this.rotationDegrees = rotationDegrees % 360.0;
        this.fixedPosition = fixedPosition;
    }

    @Override
    public String getType() {
        return "Watermark";
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
     * @return font name
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
     * @return color
     */
    public Color getTextColor() {
        return textColor;
    }

    /**
     * Gets the opacity.
     *
     * @return opacity value (0.0=transparent, 1.0=opaque)
     */
    public double getOpacity() {
        return opacity;
    }

    /**
     * Gets the rotation angle.
     *
     * @return rotation in degrees (0-360)
     */
    public double getRotationDegrees() {
        return rotationDegrees;
    }

    /**
     * Checks if watermark position is fixed.
     *
     * @return true if watermark position is fixed relative to page
     */
    public boolean isFixedPosition() {
        return fixedPosition;
    }

    /**
     * Creates a builder for watermark annotations.
     *
     * @param text watermark text
     * @return builder
     */
    public static Builder builder(String text) {
        return new Builder(text);
    }

    /**
     * Fluent builder for watermark annotations.
     */
    public static final class Builder {
        private final String contents;
        private Optional<String> author = Optional.empty();
        private Optional<Instant> createdDate = Optional.empty();
        private Optional<Instant> modifiedDate = Optional.empty();
        private Optional<String> subject = Optional.empty();
        private int flags = AnnotationFlags.PRINT;
        private String fontName = "Helvetica";
        private double fontSize = 72.0;
        private Color textColor = new Color(0.5f, 0.5f, 0.5f);  // Gray
        private double opacity = 0.3;
        private double rotationDegrees = -45.0;
        private boolean fixedPosition = true;

        private Builder(String contents) {
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
            this.textColor = color;
            return this;
        }

        /**
         * Sets the opacity.
         *
         * @param opacity opacity value (0.0-1.0)
         * @return this builder
         */
        public Builder opacity(double opacity) {
            this.opacity = Math.min(1.0, Math.max(0.0, opacity));
            return this;
        }

        /**
         * Sets the rotation angle.
         *
         * @param degrees rotation in degrees
         * @return this builder
         */
        public Builder rotation(double degrees) {
            this.rotationDegrees = degrees;
            return this;
        }

        /**
         * Sets whether position is fixed.
         *
         * @param fixed true for fixed position, false for flowing
         * @return this builder
         */
        public Builder fixedPosition(boolean fixed) {
            this.fixedPosition = fixed;
            return this;
        }

        /**
         * Builds the annotation.
         *
         * @return watermark annotation
         */
        public WatermarkAnnotation build() {
            // Watermark typically spans entire page
            Rect rect = new Rect(0, 0, 612, 792);  // Standard letter size

            return new WatermarkAnnotation(
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
                opacity,
                rotationDegrees,
                fixedPosition
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "WatermarkAnnotation('%s', font=%s %.0fpt, opacity=%.1f, rotation=%.0f°)",
            contents,
            fontName,
            fontSize,
            opacity,
            rotationDegrees
        );
    }
}
