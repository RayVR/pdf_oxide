package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for text fields.
 */
public final class TextFieldBuilder {
    private final String name;
    private final Rect rect;
    private String defaultValue;
    private int maxLength = -1;
    private boolean required = false;
    private boolean readOnly = false;

    private TextFieldBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static TextFieldBuilder create(String name, Rect rect) {
        return new TextFieldBuilder(name, rect);
    }

    public TextFieldBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public TextFieldBuilder maxLength(int length) {
        this.maxLength = length;
        return this;
    }

    public TextFieldBuilder required(boolean required) {
        this.required = required;
        return this;
    }

    public TextFieldBuilder readOnly(boolean readOnly) {
        this.readOnly = readOnly;
        return this;
    }

    public TextField build() {
        return new TextField(name, rect, defaultValue, maxLength, required, readOnly);
    }
}
