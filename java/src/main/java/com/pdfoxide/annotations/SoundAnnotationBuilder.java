package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for sound annotations.
 */
public final class SoundAnnotationBuilder {
    private final Rect rect;
    private final String audioPath;

    private SoundAnnotationBuilder(Rect rect, String audioPath) {
        this.rect = rect;
        this.audioPath = audioPath;
    }

    public static SoundAnnotationBuilder create(Rect rect, String audioPath) {
        return new SoundAnnotationBuilder(rect, audioPath);
    }

    public SoundAnnotation build() {
        return new SoundAnnotation(rect, audioPath);
    }
}
