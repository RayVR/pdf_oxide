package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Optional;

/**
 * Text field for single or multi-line text input.
 *
 * @since 1.0.0
 */
public final class TextField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final Optional<String> defaultValue;
    private final Optional<String> currentValue;
    private final Optional<Integer> maxLength;
    private final boolean multiline;
    private final boolean password;
    private final boolean fileSelect;
    private final Optional<String> fontName;
    private final Optional<Double> fontSize;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Constructs a text field.
     *
     * @param name field name
     * @param rect widget rectangle
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param defaultValue default value (optional)
     * @param currentValue current value (optional)
     * @param maxLength maximum length (optional)
     * @param multiline multi-line mode
     * @param password password mode
     * @param fileSelect file selection mode
     * @param fontName font name (optional)
     * @param fontSize font size (optional)
     * @param borderStyle border style (optional)
     */
    private TextField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            Optional<String> defaultValue,
            Optional<String> currentValue,
            Optional<Integer> maxLength,
            boolean multiline,
            boolean password,
            boolean fileSelect,
            Optional<String> fontName,
            Optional<Double> fontSize,
            Optional<BorderStyle> borderStyle) {
        this.name = name;
        this.rect = rect;
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.defaultValue = defaultValue;
        this.currentValue = currentValue;
        this.maxLength = maxLength;
        this.multiline = multiline;
        this.password = password;
        this.fileSelect = fileSelect;
        this.fontName = fontName;
        this.fontSize = fontSize;
        this.borderStyle = borderStyle;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public FormFieldType getFieldType() {
        return FormFieldType.TEXT;
    }

    @Override
    public FormFieldValue getValue() {
        return currentValue
            .map(FormFieldValue::text)
            .orElse(FormFieldValue.NULL());
    }

    @Override
    public Optional<FormFieldValue> getDefaultValue() {
        return defaultValue.map(FormFieldValue::text);
    }

    @Override
    public Rect getRect() {
        return rect;
    }

    @Override
    public Optional<String> getTooltip() {
        return tooltip;
    }

    @Override
    public boolean isReadOnly() {
        return readOnly;
    }

    @Override
    public boolean isRequired() {
        return required;
    }

    @Override
    public boolean isHidden() {
        return hidden;
    }

    @Override
    public boolean isDisabled() {
        return disabled;
    }

    public Optional<Integer> getMaxLength() {
        return maxLength;
    }

    public boolean isMultiline() {
        return multiline;
    }

    public boolean isPassword() {
        return password;
    }

    public boolean isFileSelect() {
        return fileSelect;
    }

    public Optional<String> getFontName() {
        return fontName;
    }

    public Optional<Double> getFontSize() {
        return fontSize;
    }

    public Optional<BorderStyle> getBorderStyle() {
        return borderStyle;
    }

    public static Builder builder(String name, Rect rect) {
        return new Builder(name, rect);
    }

    public static final class Builder {
        private final String name;
        private final Rect rect;
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = false;
        private boolean hidden = false;
        private boolean disabled = false;
        private Optional<String> defaultValue = Optional.empty();
        private Optional<String> currentValue = Optional.empty();
        private Optional<Integer> maxLength = Optional.empty();
        private boolean multiline = false;
        private boolean password = false;
        private boolean fileSelect = false;
        private Optional<String> fontName = Optional.empty();
        private Optional<Double> fontSize = Optional.empty();
        private Optional<BorderStyle> borderStyle = Optional.empty();

        private Builder(String name, Rect rect) {
            this.name = name;
            this.rect = rect;
        }

        public Builder tooltip(String tooltip) {
            this.tooltip = Optional.of(tooltip);
            return this;
        }

        public Builder readOnly(boolean readOnly) {
            this.readOnly = readOnly;
            return this;
        }

        public Builder required(boolean required) {
            this.required = required;
            return this;
        }

        public Builder hidden(boolean hidden) {
            this.hidden = hidden;
            return this;
        }

        public Builder disabled(boolean disabled) {
            this.disabled = disabled;
            return this;
        }

        public Builder defaultValue(String value) {
            this.defaultValue = Optional.of(value);
            return this;
        }

        public Builder value(String value) {
            this.currentValue = Optional.of(value);
            return this;
        }

        public Builder maxLength(int length) {
            this.maxLength = Optional.of(length);
            return this;
        }

        public Builder multiline(boolean multiline) {
            this.multiline = multiline;
            return this;
        }

        public Builder password(boolean password) {
            this.password = password;
            return this;
        }

        public Builder fileSelect(boolean fileSelect) {
            this.fileSelect = fileSelect;
            return this;
        }

        public Builder font(String fontName, double fontSize) {
            this.fontName = Optional.of(fontName);
            this.fontSize = Optional.of(fontSize);
            return this;
        }

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public TextField build() {
            return new TextField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                defaultValue,
                currentValue,
                maxLength,
                multiline,
                password,
                fileSelect,
                fontName,
                fontSize,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "TextField(name='%s', multiline=%s, password=%s, rect=%s)",
            name,
            multiline,
            password,
            rect
        );
    }
}
