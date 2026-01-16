package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for push button fields.
 */
public final class PushButtonBuilder {
    private final String name;
    private final Rect rect;
    private final String label;
    private ButtonAction action;

    private PushButtonBuilder(String name, Rect rect, String label) {
        this.name = name;
        this.rect = rect;
        this.label = label;
    }

    public static PushButtonBuilder create(String name, Rect rect, String label) {
        return new PushButtonBuilder(name, rect, label);
    }

    public PushButtonBuilder action(ButtonAction action) {
        this.action = action;
        return this;
    }

    public PushButtonField build() {
        return new PushButtonField(name, rect, label, action);
    }
}
