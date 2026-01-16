package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for link annotations.
 */
public final class LinkAnnotationBuilder {
    private final Rect rect;
    private final LinkAction action;

    private LinkAnnotationBuilder(Rect rect, LinkAction action) {
        this.rect = rect;
        this.action = action;
    }

    public static LinkAnnotationBuilder create(Rect rect, LinkAction action) {
        return new LinkAnnotationBuilder(rect, action);
    }

    public LinkAnnotation build() {
        return new LinkAnnotation(rect, action);
    }
}
