package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Base class for all PDF annotations.
 *
 * <p>Annotations are interactive elements that can be added to PDF pages, such as
 * comments, highlights, links, and form fields. All annotations have a location
 * (rectangle), content, and optional metadata.
 *
 * @since 1.0.0
 */
public abstract class Annotation {
    protected final Rect rect;
    protected String contents = "";
    protected Optional<String> author = Optional.empty();
    protected Optional<Instant> createdDate = Optional.empty();
    protected Optional<Instant> modifiedDate = Optional.empty();
    protected Optional<String> subject = Optional.empty();
    protected int flags = 0;

    protected Annotation(Rect rect) {
        this.rect = rect;
    }

    protected Annotation(Rect rect, String contents) {
        this.rect = rect;
        this.contents = contents;
    }

    /**
     * Gets the annotation type.
     *
     * @return annotation type name (e.g., "Text", "Highlight", "Link")
     */
    public abstract String getType();

    /**
     * Gets the location and size of the annotation on the page.
     *
     * @return bounding rectangle
     */
    public Rect getRect() {
        return rect;
    }

    /**
     * Gets the annotation's content or label.
     *
     * @return content text
     */
    public String getContents() {
        return contents;
    }

    /**
     * Gets the name of the user/application that created the annotation.
     *
     * @return author name, empty if not set
     */
    public Optional<String> getAuthor() {
        return author;
    }

    /**
     * Gets the creation date.
     *
     * @return creation timestamp, empty if not set
     */
    public Optional<Instant> getCreatedDate() {
        return createdDate;
    }

    /**
     * Gets the last modification date.
     *
     * @return modification timestamp, empty if not set
     */
    public Optional<Instant> getModifiedDate() {
        return modifiedDate;
    }

    /**
     * Gets the annotation subject/topic.
     *
     * @return subject text, empty if not set
     */
    public Optional<String> getSubject() {
        return subject;
    }

    /**
     * Gets the annotation's display flags.
     *
     * @return combination of AnnotationFlags constants
     */
    public int getFlags() {
        return flags;
    }

    /**
     * Checks if a specific flag is set.
     *
     * @param flag flag constant from AnnotationFlags
     * @return true if flag is set
     */
    public boolean hasFlag(int flag) {
        return AnnotationFlags.hasFlag(getFlags(), flag);
    }

    /**
     * Checks if the annotation is visible.
     *
     * @return true if not marked as invisible or hidden
     */
    public boolean isVisible() {
        return !hasFlag(AnnotationFlags.INVISIBLE) && !hasFlag(AnnotationFlags.HIDDEN);
    }

    /**
     * Checks if the annotation is printed.
     *
     * @return true if PRINT flag is set
     */
    public boolean isPrintable() {
        return hasFlag(AnnotationFlags.PRINT);
    }

    /**
     * Checks if the annotation is read-only.
     *
     * @return true if READ_ONLY flag is set
     */
    public boolean isReadOnly() {
        return hasFlag(AnnotationFlags.READ_ONLY);
    }

    /**
     * Checks if the annotation is locked.
     *
     * @return true if LOCKED flag is set
     */
    public boolean isLocked() {
        return hasFlag(AnnotationFlags.LOCKED);
    }
}
