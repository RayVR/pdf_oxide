package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Arrays;
import java.util.List;

/**
 * Builder for list box fields.
 */
public final class ListBoxBuilder {
    private final String name;
    private final Rect rect;
    private List<String> options;
    private boolean multiSelect = false;
    private List<String> selectedValues;

    private ListBoxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static ListBoxBuilder create(String name, Rect rect) {
        return new ListBoxBuilder(name, rect);
    }

    public ListBoxBuilder options(String... options) {
        this.options = Arrays.asList(options);
        return this;
    }

    public ListBoxBuilder options(List<String> options) {
        this.options = options;
        return this;
    }

    public ListBoxBuilder multiSelect(boolean multiSelect) {
        this.multiSelect = multiSelect;
        return this;
    }

    public ListBoxBuilder selectedValues(List<String> values) {
        this.selectedValues = values;
        return this;
    }

    public ListBoxField build() {
        return new ListBoxField(name, rect, options, multiSelect, selectedValues);
    }
}
