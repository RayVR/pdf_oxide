package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Arrays;
import java.util.List;

/**
 * Builder for combo box fields.
 */
public final class ComboBoxBuilder {
    private final String name;
    private final Rect rect;
    private List<String> options;
    private boolean editable = false;
    private String defaultValue;

    private ComboBoxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static ComboBoxBuilder create(String name, Rect rect) {
        return new ComboBoxBuilder(name, rect);
    }

    public ComboBoxBuilder options(String... options) {
        this.options = Arrays.asList(options);
        return this;
    }

    public ComboBoxBuilder options(List<String> options) {
        this.options = options;
        return this;
    }

    public ComboBoxBuilder editable(boolean editable) {
        this.editable = editable;
        return this;
    }

    public ComboBoxBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public ComboBoxField build() {
        return new ComboBoxField(name, rect, options, editable, defaultValue);
    }
}
