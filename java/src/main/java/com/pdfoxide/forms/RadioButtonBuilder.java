package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Helper for radio button definitions.
 */
public final class RadioButtonBuilder {
    public static final class Option {
        private final Rect rect;
        private final String value;

        public Option(Rect rect, String value) {
            this.rect = rect;
            this.value = value;
        }

        public Rect getRect() {
            return rect;
        }

        public String getValue() {
            return value;
        }
    }
}
