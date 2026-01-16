package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Base interface for all PDF annotations.
 *
 * <p>Annotations are interactive elements that can be added to PDF pages, such as
 * comments, highlights, links, and form fields. All annotations have a location
 * (rectangle), content, and optional metadata.
 *
 * @since 1.0.0
 */
public interface Annotation {

    /**
     * Gets the annotation type.
     *
     * @return annotation type name (e.g., "Text", "Highlight", "Link")
     */
    String getType();

    /**
     * Gets the location and size of the annotation on the page.
     *
     * @return bounding rectangle
     */
    Rect getRect();

    /**
     * Gets the annotation's content or label.
     *
     * @return content text
     */
    String getContents();

    /**
     * Gets the name of the user/application that created the annotation.
     *
     * @return author name, empty if not set
     */
    Optional<String> getAuthor();

    /**
     * Gets the creation date.
     *
     * @return creation timestamp, empty if not set
     */
    Optional<Instant> getCreatedDate();

    /**
     * Gets the last modification date.
     *
     * @return modification timestamp, empty if not set
     */
    Optional<Instant> getModifiedDate();

    /**
     * Gets the annotation subject/topic.
     *
     * @return subject text, empty if not set
     */
    Optional<String> getSubject();

    /**
     * Gets the annotation's display flags.
     *
     * @return combination of AnnotationFlags constants
     */
    int getFlags();

    /**
     * Checks if a specific flag is set.
     *
     * @param flag flag constant from AnnotationFlags
     * @return true if flag is set
     */
    default boolean hasFlag(int flag) {
        return AnnotationFlags.hasFlag(getFlags(), flag);
    }

    /**
     * Checks if the annotation is visible.
     *
     * @return true if not marked as invisible or hidden
     */
    default boolean isVisible() {
        return !hasFlag(AnnotationFlags.INVISIBLE) && !hasFlag(AnnotationFlags.HIDDEN);
    }

    /**
     * Checks if the annotation is printed.
     *
     * @return true if PRINT flag is set
     */
    default boolean isPrintable() {
        return hasFlag(AnnotationFlags.PRINT);
    }

    /**
     * Checks if the annotation is read-only.
     *
     * @return true if READ_ONLY flag is set
     */
    default boolean isReadOnly() {
        return hasFlag(AnnotationFlags.READ_ONLY);
    }

    /**
     * Checks if the annotation is locked.
     *
     * @return true if LOCKED flag is set
     */
    default boolean isLocked() {
        return hasFlag(AnnotationFlags.LOCKED);
    }
}
