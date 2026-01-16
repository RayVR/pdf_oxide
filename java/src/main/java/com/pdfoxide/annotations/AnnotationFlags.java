package com.pdfoxide.annotations;

/**
 * Standard PDF annotation flags.
 *
 * <p>These flags control annotation appearance and behavior according to the PDF specification.
 *
 * @since 1.0.0
 */
public final class AnnotationFlags {
    private AnnotationFlags() {}

    /** Annotation is invisible. */
    public static final int INVISIBLE = 1;

    /** Annotation is hidden and should not be displayed or printed. */
    public static final int HIDDEN = 2;

    /** Annotation is printed. */
    public static final int PRINT = 4;

    /** Annotation does not zoom or rotate with the page. */
    public static final int NO_ZOOM = 8;

    /** Annotation does not rotate with the page. */
    public static final int NO_ROTATE = 16;

    /** Annotation is not interacted with. */
    public static final int NO_VIEW = 32;

    /** Annotation is read-only. */
    public static final int READ_ONLY = 64;

    /** Annotation is locked (cannot be deleted or properties modified). */
    public static final int LOCKED = 128;

    /** Annotation is toggled on/off when clicked. */
    public static final int TOGGLE_NO_VIEW = 256;

    /** Annotation is locked for content. */
    public static final int LOCKED_CONTENTS = 512;

    /** Combine flags bitwise OR. Example: PRINT | READ_ONLY */
    public static int combine(int... flags) {
        int result = 0;
        for (int flag : flags) {
            result |= flag;
        }
        return result;
    }

    /** Check if a flag is set. */
    public static boolean hasFlag(int flags, int flag) {
        return (flags & flag) != 0;
    }
}
