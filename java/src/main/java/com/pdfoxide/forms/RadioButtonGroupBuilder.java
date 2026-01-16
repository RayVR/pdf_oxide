package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.ArrayList;
import java.util.List;

/**
 * Builder for radio button groups.
 */
public final class RadioButtonGroupBuilder {
    private final String name;
    private final List<RadioButtonBuilder.Option> options = new ArrayList<>();
    private String defaultValue;

    private RadioButtonGroupBuilder(String name) {
        this.name = name;
    }

    public static RadioButtonGroupBuilder create(String name) {
        return new RadioButtonGroupBuilder(name);
    }

    public RadioButtonGroupBuilder addButton(Rect rect, String value) {
        options.add(new RadioButtonBuilder.Option(rect, value));
        return this;
    }

    public RadioButtonGroupBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public RadioButtonField build() {
        return new RadioButtonField(name, options, defaultValue);
    }
}
