package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for movie annotations.
 */
public final class MovieAnnotationBuilder {
    private final Rect rect;
    private final String moviePath;

    private MovieAnnotationBuilder(Rect rect, String moviePath) {
        this.rect = rect;
        this.moviePath = moviePath;
    }

    public static MovieAnnotationBuilder create(Rect rect, String moviePath) {
        return new MovieAnnotationBuilder(rect, moviePath);
    }

    public MovieAnnotation build() {
        return new MovieAnnotation(rect, moviePath);
    }
}
