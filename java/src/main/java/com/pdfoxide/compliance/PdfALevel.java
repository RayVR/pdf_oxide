package com.pdfoxide.compliance;

/**
 * PDF/A conformance level.
 *
 * <p>Specifies the level of PDF/A compliance to validate against.
 *
 * @since 1.0.0
 */
public enum PdfALevel {
    /**
     * PDF/A-1b: Aims at visual appearance and long-term preservation.
     * Basic conformance level with visual reproduction as primary goal.
     */
    LEVEL_1B("1B"),

    /**
     * PDF/A-1a: Adds tagged PDF (logical structure) requirements.
     * Full conformance with both visual appearance and semantic structure.
     */
    LEVEL_1A("1A"),

    /**
     * PDF/A-2b: Based on PDF 1.7 with enhanced features.
     * Basic conformance level for PDF 1.7 documents.
     */
    LEVEL_2B("2B"),

    /**
     * PDF/A-2a: Tagged PDF variant of PDF/A-2.
     * Full conformance with logical structure for PDF 1.7.
     */
    LEVEL_2A("2A"),

    /**
     * PDF/A-2u: Unicode variant emphasizing character preservation.
     * Unicode-based conformance for PDF 1.7.
     */
    LEVEL_2U("2U"),

    /**
     * PDF/A-3b: Based on PDF 2.0 with embedded file support.
     * Basic conformance level for PDF 2.0 documents.
     */
    LEVEL_3B("3B"),

    /**
     * PDF/A-3a: Tagged PDF variant of PDF/A-3.
     * Full conformance with logical structure for PDF 2.0.
     */
    LEVEL_3A("3A"),

    /**
     * PDF/A-3u: Unicode variant of PDF/A-3.
     * Unicode-based conformance for PDF 2.0.
     */
    LEVEL_3U("3U");

    private final String code;

    PdfALevel(String code) {
        this.code = code;
    }

    /**
     * Gets the level code (e.g., "1B", "2A").
     *
     * @return level code string
     */
    public String getCode() {
        return code;
    }

    /**
     * Gets the major version (1, 2, or 3).
     *
     * @return major version
     */
    public int getMajorVersion() {
        return Integer.parseInt(code.substring(0, 1));
    }

    /**
     * Gets the level variant ('A', 'B', or 'U').
     *
     * @return level variant character
     */
    public char getVariant() {
        return code.charAt(1);
    }

    /**
     * Checks if this level requires tagged PDF.
     *
     * @return true for 'a' variant levels, false otherwise
     */
    public boolean requiresTaggedPdf() {
        return getVariant() == 'A';
    }

    /**
     * Checks if this level emphasizes Unicode.
     *
     * @return true for 'u' variant levels, false otherwise
     */
    public boolean isUnicodeVariant() {
        return getVariant() == 'U';
    }

    /**
     * Parses a PDF/A level from a string.
     *
     * @param code level code (e.g., "1B", "2A")
     * @return corresponding PdfALevel
     * @throws IllegalArgumentException if code is not valid
     */
    public static PdfALevel parse(String code) {
        for (PdfALevel level : values()) {
            if (level.code.equalsIgnoreCase(code)) {
                return level;
            }
        }
        throw new IllegalArgumentException("Invalid PDF/A level: " + code);
    }
}
