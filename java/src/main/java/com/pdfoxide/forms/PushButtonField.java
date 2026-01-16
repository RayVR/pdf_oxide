package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Optional;

/**
 * Push button field for triggering actions.
 *
 * @since 1.0.0
 */
public final class PushButtonField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final String caption;
    private final ButtonAction action;
    private final Optional<String> iconName;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Button action type.
     */
    public enum ButtonAction {
        SUBMIT,         // Submit form
        RESET,          // Reset form
        IMPORT,         // Import form data
        JAVASCRIPT,     // Execute JavaScript
        NONE            // No action
    }

    /**
     * Constructs a push button field.
     *
     * @param name field name
     * @param rect widget rectangle
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param caption button label
     * @param action action on click
     * @param iconName icon appearance (optional)
     * @param borderStyle border style (optional)
     */
    private PushButtonField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            String caption,
            ButtonAction action,
            Optional<String> iconName,
            Optional<BorderStyle> borderStyle) {
        this.name = name;
        this.rect = rect;
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.caption = caption;
        this.action = action;
        this.iconName = iconName;
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
        return FormFieldValue.NULL();
    }

    @Override
    public Optional<FormFieldValue> getDefaultValue() {
        return Optional.empty();
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

    public String getCaption() {
        return caption;
    }

    public ButtonAction getAction() {
        return action;
    }

    public Optional<String> getIconName() {
        return iconName;
    }

    public Optional<BorderStyle> getBorderStyle() {
        return borderStyle;
    }

    public static Builder builder(String name, Rect rect, String caption, ButtonAction action) {
        return new Builder(name, rect, caption, action);
    }

    public static final class Builder {
        private final String name;
        private final Rect rect;
        private final String caption;
        private final ButtonAction action;
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = false;
        private boolean hidden = false;
        private boolean disabled = false;
        private Optional<String> iconName = Optional.empty();
        private Optional<BorderStyle> borderStyle = Optional.empty();

        private Builder(String name, Rect rect, String caption, ButtonAction action) {
            this.name = name;
            this.rect = rect;
            this.caption = caption;
            this.action = action;
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

        public Builder iconName(String iconName) {
            this.iconName = Optional.of(iconName);
            return this;
        }

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public PushButtonField build() {
            return new PushButtonField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                caption,
                action,
                iconName,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "PushButtonField(name='%s', caption='%s', action=%s, rect=%s)",
            name,
            caption,
            action,
            rect
        );
    }
}
