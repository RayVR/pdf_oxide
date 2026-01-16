package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for rich media annotations.
 */
public final class RichMediaAnnotationBuilder {
    private final Rect rect;
    private final String mediaPath;

    private RichMediaAnnotationBuilder(Rect rect, String mediaPath) {
        this.rect = rect;
        this.mediaPath = mediaPath;
    }

    public static RichMediaAnnotationBuilder create(Rect rect, String mediaPath) {
        return new RichMediaAnnotationBuilder(rect, mediaPath);
    }

    public RichMediaAnnotation build() {
        return new RichMediaAnnotation(rect, mediaPath);
    }
}
