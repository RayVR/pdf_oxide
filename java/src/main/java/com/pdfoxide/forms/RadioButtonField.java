package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * Radio button field for mutually exclusive choices.
 *
 * @since 1.0.0
 */
public final class RadioButtonField implements FormField {
    private final String name;
    private final List<Rect> buttonRects;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final List<String> options;
    private final Optional<String> defaultValue;
    private final Optional<String> currentValue;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Constructs a radio button field.
     *
     * @param name field name
     * @param buttonRects rectangles for each radio button
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param options export values for each button
     * @param defaultValue default selection (optional)
     * @param currentValue current selection (optional)
     * @param borderStyle border style (optional)
     */
    private RadioButtonField(
            String name,
            List<Rect> buttonRects,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            List<String> options,
            Optional<String> defaultValue,
            Optional<String> currentValue,
            Optional<BorderStyle> borderStyle) {
        this.name = name;
        this.buttonRects = Collections.unmodifiableList(buttonRects);
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.options = Collections.unmodifiableList(options);
        this.defaultValue = defaultValue;
        this.currentValue = currentValue;
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
        // Return bounding rect of all buttons
        if (buttonRects.isEmpty()) {
            return new Rect(0, 0, 0, 0);
        }
        float minX = Float.MAX_VALUE, minY = Float.MAX_VALUE;
        float maxX = Float.MIN_VALUE, maxY = Float.MIN_VALUE;

        for (Rect rect : buttonRects) {
            minX = Math.min(minX, rect.getX());
            minY = Math.min(minY, rect.getY());
            maxX = Math.max(maxX, rect.getRight());
            maxY = Math.max(maxY, rect.getTop());
        }

        return new Rect(minX, minY, maxX - minX, maxY - minY);
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

    public List<Rect> getButtonRects() {
        return buttonRects;
    }

    public int getButtonCount() {
        return buttonRects.size();
    }

    public List<String> getOptions() {
        return options;
    }

    public Optional<BorderStyle> getBorderStyle() {
        return borderStyle;
    }

    public static Builder builder(String name) {
        return new Builder(name);
    }

    public static final class Builder {
        private final String name;
        private final java.util.List<Rect> buttonRects = new java.util.ArrayList<>();
        private final java.util.List<String> options = new java.util.ArrayList<>();
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = false;
        private boolean hidden = false;
        private boolean disabled = false;
        private Optional<String> defaultValue = Optional.empty();
        private Optional<String> currentValue = Optional.empty();
        private Optional<BorderStyle> borderStyle = Optional.empty();

        private Builder(String name) {
            this.name = name;
        }

        public Builder button(Rect rect, String exportValue) {
            buttonRects.add(rect);
            options.add(exportValue);
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

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public RadioButtonField build() {
            if (buttonRects.isEmpty()) {
                throw new IllegalStateException("RadioButton must have at least one button");
            }

            return new RadioButtonField(
                name,
                buttonRects,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                options,
                defaultValue,
                currentValue,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "RadioButtonField(name='%s', buttons=%d, selected=%s, rect=%s)",
            name,
            getButtonCount(),
            currentValue.orElse("(none)"),
            getRect()
        );
    }
}
