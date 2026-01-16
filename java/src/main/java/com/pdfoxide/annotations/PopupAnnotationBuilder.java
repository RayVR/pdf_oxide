package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for popup annotations.
 */
public final class PopupAnnotationBuilder {
    private final Rect rect;
    private final String content;

    private PopupAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static PopupAnnotationBuilder create(Rect rect, String content) {
        return new PopupAnnotationBuilder(rect, content);
    }

    public PopupAnnotation build() {
        return new PopupAnnotation(rect, content);
    }
}
