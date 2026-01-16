package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

/**
 * List box field for single or multi-select choices.
 *
 * @since 1.0.0
 */
public final class ListBoxField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final List<String> options;
    private final List<String> displayOptions;
    private final List<String> selectedValues;
    private final boolean multiSelect;
    private final Optional<Integer> topIndex;
    private final Optional<BorderStyle> borderStyle;

    /**
     * Constructs a list box field.
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
     * @param selectedValues currently selected values
     * @param multiSelect allow multiple selections
     * @param topIndex first visible option index (optional)
     * @param borderStyle border style (optional)
     */
    private ListBoxField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            List<String> options,
            List<String> displayOptions,
            List<String> selectedValues,
            boolean multiSelect,
            Optional<Integer> topIndex,
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
        this.selectedValues = Collections.unmodifiableList(selectedValues);
        this.multiSelect = multiSelect;
        this.topIndex = topIndex;
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
        return selectedValues.isEmpty()
            ? FormFieldValue.NULL()
            : selectedValues.size() == 1
                ? FormFieldValue.name(selectedValues.get(0))
                : FormFieldValue.array(selectedValues);
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

    public List<String> getOptions() {
        return options;
    }

    public List<String> getDisplayOptions() {
        return displayOptions;
    }

    public int getOptionCount() {
        return options.size();
    }

    public List<String> getSelectedValues() {
        return selectedValues;
    }

    public int getSelectionCount() {
        return selectedValues.size();
    }

    public boolean isMultiSelect() {
        return multiSelect;
    }

    public Optional<Integer> getTopIndex() {
        return topIndex;
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
        private java.util.List<String> selectedValues = new java.util.ArrayList<>();
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = false;
        private boolean hidden = false;
        private boolean disabled = false;
        private boolean multiSelect = false;
        private Optional<Integer> topIndex = Optional.empty();
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

        public Builder select(String value) {
            selectedValues.add(value);
            return this;
        }

        public Builder selected(List<String> values) {
            this.selectedValues = new java.util.ArrayList<>(values);
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

        public Builder multiSelect(boolean multiSelect) {
            this.multiSelect = multiSelect;
            return this;
        }

        public Builder topIndex(int index) {
            this.topIndex = Optional.of(index);
            return this;
        }

        public Builder borderStyle(BorderStyle style) {
            this.borderStyle = Optional.of(style);
            return this;
        }

        public ListBoxField build() {
            if (options.isEmpty()) {
                throw new IllegalStateException("ListBox must have at least one option");
            }

            return new ListBoxField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                options,
                displayOptions,
                selectedValues,
                multiSelect,
                topIndex,
                borderStyle
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "ListBoxField(name='%s', options=%d, selected=%d, multiselect=%s, rect=%s)",
            name,
            getOptionCount(),
            getSelectionCount(),
            multiSelect,
            rect
        );
    }
}
