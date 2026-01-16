package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for checkbox fields.
 */
public final class CheckboxBuilder {
    private final String name;
    private final Rect rect;
    private boolean defaultChecked = false;
    private String exportValue = "Yes";

    private CheckboxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static CheckboxBuilder create(String name, Rect rect) {
        return new CheckboxBuilder(name, rect);
    }

    public CheckboxBuilder defaultChecked(boolean checked) {
        this.defaultChecked = checked;
        return this;
    }

    public CheckboxBuilder exportValue(String value) {
        this.exportValue = value;
        return this;
    }

    public CheckboxField build() {
        return new CheckboxField(name, rect, defaultChecked, exportValue);
    }
}
