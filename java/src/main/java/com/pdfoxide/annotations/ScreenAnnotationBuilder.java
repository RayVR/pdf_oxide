package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for screen annotations.
 */
public final class ScreenAnnotationBuilder {
    private final Rect rect;
    private final String mediaPath;

    private ScreenAnnotationBuilder(Rect rect, String mediaPath) {
        this.rect = rect;
        this.mediaPath = mediaPath;
    }

    public static ScreenAnnotationBuilder create(Rect rect, String mediaPath) {
        return new ScreenAnnotationBuilder(rect, mediaPath);
    }

    public ScreenAnnotation build() {
        return new ScreenAnnotation(rect, mediaPath);
    }
}
