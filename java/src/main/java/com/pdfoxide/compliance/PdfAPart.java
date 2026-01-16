package com.pdfoxide.compliance;

/**
 * PDF/A part specification.
 *
 * <p>Specifies which part of the PDF/A specification to validate against.
 *
 * @since 1.0.0
 */
public enum PdfAPart {
    /**
     * PDF/A Part 1: Base specification for PDF/A conformance.
     * Based on PDF 1.4 with restrictions for long-term preservation.
     */
    PART_1,

    /**
     * PDF/A Part 2: Enhanced specification based on PDF 1.7.
     * Adds support for transparency, 3D content, and other PDF 1.7 features.
     */
    PART_2,

    /**
     * PDF/A Part 3: Extended specification based on PDF 2.0.
     * Adds support for embedded files and other PDF 2.0 features.
     */
    PART_3,

    /**
     * PDF/A Part 4: Experimental/future specification (not yet standardized).
     * Future extensions to PDF/A conformance.
     */
    PART_4;

    /**
     * Gets the part number (1, 2, 3, or 4).
     *
     * @return part number
     */
    public int getPartNumber() {
        switch (this) {
            case PART_1: return 1;
            case PART_2: return 2;
            case PART_3: return 3;
            case PART_4: return 4;
            default: return 0;
        }
    }

    /**
     * Gets the PDF version this part is based on.
     *
     * @return PDF version string (e.g., "1.4", "1.7", "2.0")
     */
    public String getBasePdfVersion() {
        switch (this) {
            case PART_1: return "1.4";
            case PART_2: return "1.7";
            case PART_3: return "2.0";
            case PART_4: return "2.0+";
            default: return "unknown";
        }
    }

    /**
     * Parses a PDF/A part from a string.
     *
     * @param partString part identifier (e.g., "1", "2", "3", "4")
     * @return corresponding PdfAPart
     * @throws IllegalArgumentException if part is not valid
     */
    public static PdfAPart parse(String partString) {
        switch (partString) {
            case "1": return PART_1;
            case "2": return PART_2;
            case "3": return PART_3;
            case "4": return PART_4;
            default: throw new IllegalArgumentException("Invalid PDF/A part: " + partString);
        }
    }
}
