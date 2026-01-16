package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Optional;

/**
 * Checkbox field for boolean selection.
 *
 * @since 1.0.0
 */
public final class CheckboxField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final boolean defaultChecked;
    private final boolean currentChecked;
    private final Optional<String> exportValue;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Constructs a checkbox field.
     *
     * @param name field name
     * @param rect widget rectangle
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param defaultChecked default checked state
     * @param currentChecked current checked state
     * @param exportValue export value (optional)
     * @param borderStyle border style (optional)
     */
    private CheckboxField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            boolean defaultChecked,
            boolean currentChecked,
            Optional<String> exportValue,
            Optional<BorderStyle> borderStyle) {
        this.name = name;
        this.rect = rect;
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.defaultChecked = defaultChecked;
        this.currentChecked = currentChecked;
        this.exportValue = exportValue;
        this.borderStyle = borderStyle;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public FormFieldType getFieldType() {
        return FormFieldType.BUTTON;
    }

    @Override
    public FormFieldValue getValue() {
        return FormFieldValue.bool(currentChecked);
    }

    @Override
    public Optional<FormFieldValue> getDefaultValue() {
        return Optional.of(FormFieldValue.bool(defaultChecked));
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

    public boolean isChecked() {
        return currentChecked;
    }

    public boolean isDefaultChecked() {
        return defaultChecked;
    }

    public Optional<String> getExportValue() {
        return exportValue;
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
        private boolean defaultChecked = false;
        private boolean currentChecked = false;
        private Optional<String> exportValue = Optional.empty();
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

        public Builder defaultChecked(boolean checked) {
            this.defaultChecked = checked;
            return this;
        }

        public Builder checked(boolean checked) {
            this.currentChecked = checked;
            return this;
        }

        public Builder exportValue(String value) {
            this.exportValue = Optional.of(value);
            return this;
        }

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public CheckboxField build() {
            return new CheckboxField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                defaultChecked,
                currentChecked,
                exportValue,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format("CheckboxField(name='%s', checked=%s, rect=%s)", name, currentChecked, rect);
    }
}
