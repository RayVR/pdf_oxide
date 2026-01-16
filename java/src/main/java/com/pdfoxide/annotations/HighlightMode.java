package com.pdfoxide.annotations;

/**
 * Highlight annotation modes.
 *
 * <p>Specifies how text is highlighted in a highlight annotation.
 *
 * @since 1.0.0
 */
public enum HighlightMode {
    /** Full highlight (yellow background). */
    HIGHLIGHT,

    /** Underline text (bottom line). */
    UNDERLINE,

    /** Strikeout text (line through). */
    STRIKEOUT,

    /** Squiggly underline (wavy line). */
    SQUIGGLY;

    public int toNativeOrdinal() {
        return ordinal();
    }

    public static HighlightMode fromNativeOrdinal(int ordinal) {
        return values()[ordinal];
    }
}
