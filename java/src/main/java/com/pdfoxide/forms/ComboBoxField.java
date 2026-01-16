package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Combo box field (dropdown with optional text entry).
 *
 * @since 1.0.0
 */
public final class ComboBoxField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final List<String> options;
    private final List<String> displayOptions;
    private final Optional<String> defaultValue;
    private final Optional<String> currentValue;
    private final boolean editable;
    private final Optional<Integer> maxLength;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Constructs a combo box field.
     *
     * @param name field name
     * @param rect widget rectangle
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param options export values
     * @param displayOptions display names
     * @param defaultValue default selection (optional)
     * @param currentValue current selection (optional)
     * @param editable user can enter custom values
     * @param maxLength maximum text length (optional)
     * @param borderStyle border style (optional)
     */
    private ComboBoxField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            List<String> options,
            List<String> displayOptions,
            Optional<String> defaultValue,
            Optional<String> currentValue,
            boolean editable,
            Optional<Integer> maxLength,
            Optional<BorderStyle> borderStyle) {
        this.name = name;
        this.rect = rect;
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.options = Collections.unmodifiableList(options);
        this.displayOptions = Collections.unmodifiableList(displayOptions);
        this.defaultValue = defaultValue;
        this.currentValue = currentValue;
        this.editable = editable;
        this.maxLength = maxLength;
        this.borderStyle = borderStyle;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public FormFieldType getFieldType() {
        return FormFieldType.CHOICE;
    }

    @Override
    public FormFieldValue getValue() {
        return currentValue
            .map(FormFieldValue::name)
            .orElse(FormFieldValue.NULL());
    }

    @Override
    public Optional<FormFieldValue> getDefaultValue() {
        return defaultValue.map(FormFieldValue::name);
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

    public List<String> getOptions() {
        return options;
    }

    public List<String> getDisplayOptions() {
        return displayOptions;
    }

    public int getOptionCount() {
        return options.size();
    }

    public boolean isEditable() {
        return editable;
    }

    public Optional<Integer> getMaxLength() {
        return maxLength;
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
        private java.util.List<String> options = new java.util.ArrayList<>();
        private java.util.List<String> displayOptions = new java.util.ArrayList<>();
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = false;
        private boolean hidden = false;
        private boolean disabled = false;
        private Optional<String> defaultValue = Optional.empty();
        private Optional<String> currentValue = Optional.empty();
        private boolean editable = false;
        private Optional<Integer> maxLength = Optional.empty();
        private Optional<BorderStyle> borderStyle = Optional.empty();

        private Builder(String name, Rect rect) {
            this.name = name;
            this.rect = rect;
        }

        public Builder option(String displayText) {
            return option(displayText, displayText);
        }

        public Builder option(String displayText, String exportValue) {
            displayOptions.add(displayText);
            options.add(exportValue);
            return this;
        }

        public Builder options(List<String> opts) {
            return options(opts, opts);
        }

        public Builder options(List<String> displayNames, List<String> exportValues) {
            if (displayNames.size() != exportValues.size()) {
                throw new IllegalArgumentException("Display names and export values must have same size");
            }
            this.displayOptions = new java.util.ArrayList<>(displayNames);
            this.options = new java.util.ArrayList<>(exportValues);
            return this;
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

        public Builder editable(boolean editable) {
            this.editable = editable;
            return this;
        }

        public Builder maxLength(int length) {
            this.maxLength = Optional.of(length);
            return this;
        }

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public ComboBoxField build() {
            if (options.isEmpty()) {
                throw new IllegalStateException("ComboBox must have at least one option");
            }

            return new ComboBoxField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                options,
                displayOptions,
                defaultValue,
                currentValue,
                editable,
                maxLength,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "ComboBoxField(name='%s', options=%d, editable=%s, rect=%s)",
            name,
            getOptionCount(),
            editable,
            rect
        );
    }
}
