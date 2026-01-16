package com.pdfoxide.annotations;

/**
 * Standard stamp types for stamp annotations.
 *
 * <p>Represents predefined stamp appearances in PDF documents.
 *
 * @since 1.0.0
 */
public enum StampType {
    /** "APPROVED" stamp. */
    APPROVED,

    /** "AS IS" stamp. */
    AS_IS,

    /** "EXPIRED" stamp. */
    EXPIRED,

    /** "NOT APPROVED" stamp. */
    NOT_APPROVED,

    /** "NOT FOR PUBLIC RELEASE" stamp. */
    NOT_FOR_PUBLIC_RELEASE,

    /** "CONFIDENTIAL" stamp. */
    CONFIDENTIAL,

    /** "TOP SECRET" stamp. */
    TOP_SECRET,

    /** "FOR COMMENT" stamp. */
    FOR_COMMENT,

    /** "DRAFT" stamp. */
    DRAFT;

    public int toNativeOrdinal() {
        return ordinal();
    }

    public static StampType fromNativeOrdinal(int ordinal) {
        return values()[ordinal];
    }
}
