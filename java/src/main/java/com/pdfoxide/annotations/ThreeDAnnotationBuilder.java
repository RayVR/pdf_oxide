package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for 3D annotations.
 */
public final class ThreeDAnnotationBuilder {
    private final Rect rect;
    private final String u3dPath;

    private ThreeDAnnotationBuilder(Rect rect, String u3dPath) {
        this.rect = rect;
        this.u3dPath = u3dPath;
    }

    public static ThreeDAnnotationBuilder create(Rect rect, String u3dPath) {
        return new ThreeDAnnotationBuilder(rect, u3dPath);
    }

    public ThreeDAnnotation build() {
        return new ThreeDAnnotation(rect, u3dPath);
    }
}
