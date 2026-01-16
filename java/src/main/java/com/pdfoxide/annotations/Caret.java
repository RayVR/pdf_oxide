package com.pdfoxide.annotations;

/**
 * Caret symbol types.
 */
public enum Caret {
    P, // Paragraph
    NONE;

    @Override
    public String toString() {
        return name().equals("NONE") ? "None" : name();
    }
}
