package com.pdfoxide.compliance;

/**
 * PDF/A validation warning codes.
 *
 * <p>Enumeration of warning conditions that indicate potential issues with PDF/A
 * compliance. Warnings are non-critical concerns that may affect interoperability
 * or long-term preservation but do not prevent validation success.
 *
 * @since 1.0.0
 */
public enum WarningCode {
    // Font warnings
    FONT_SUBSTITUTION_POSSIBLE("Font may be substituted by viewer"),
    SUBSET_FONT_USED("Subset font may not be fully supported"),
    STANDARD_FONT_USED("Standard font used instead of embedded"),
    FONT_FILE_MISSING("Font file reference incomplete"),

    // Color warnings
    UNCALIBRATED_COLOR_SPACE("Color space may be uncalibrated"),
    RGB_COLOR_SPACE_USED("RGB color space used (CMYK recommended)"),
    DEVICE_DEPENDENT_COLOR("Device-dependent color may not preserve accurately"),

    // Content warnings
    EXTERNAL_LINK_USED("External link may not be accessible"),
    SCRIPT_ACTION_USED("Script action may not execute in PDF/A viewer"),
    FORM_FIELD_WITHOUT_EXPORT_VALUE("Form field missing export value"),
    ANNOTATION_WITHOUT_DATE("Annotation missing creation date"),

    // Metadata warnings
    MISSING_XMP_METADATA("XMP metadata is missing"),
    INCOMPLETE_DOCUMENT_INFO("Document information is incomplete"),
    AUTHOR_NOT_SPECIFIED("Author information not specified"),
    TITLE_NOT_SPECIFIED("Document title not specified"),
    SUBJECT_NOT_SPECIFIED("Subject not specified"),

    // Image warnings
    LOW_RESOLUTION_IMAGE("Image has low resolution (< 150 dpi)"),
    UNCOMPRESSED_IMAGE("Uncompressed image may increase file size"),
    IMAGE_WITHOUT_INTERPOLATION("Image interpolation not specified"),
    JPEG2000_USED("JPEG2000 compression may have limited support"),

    // Structure warnings
    INCOMPLETE_STRUCTURE_TREE("Structure tree may be incomplete"),
    UNMARKED_CONTENT("Some content is not marked in structure tree"),
    MISSING_ROLE_MAP("Role map is missing for tagged content"),

    // Transparency warnings
    LIGHT_TRANSPARENCY_USED("Light transparency used"),
    COMPLEX_BLEND_MODE("Complex blend mode may not render identically"),

    // File attachment warnings
    LARGE_FILE_ATTACHMENT("Large file attachment may affect performance"),
    BINARY_FILE_ATTACHED("Binary file attached (text recommended)"),

    // Preservation warnings
    MODERN_PDF_FEATURE_USED("Modern PDF feature may not be preserved long-term"),
    OPTIONAL_CONTENT_USED("Optional content may not be supported"),

    // Other warnings
    UNUSUAL_DOCUMENT_STRUCTURE("Unusual document structure detected"),
    PERFORMANCE_OPTIMIZATION_POSSIBLE("Document could be optimized for performance"),
    FILE_SIZE_WARNING("File size is unusually large for a PDF/A document");

    private final String description;

    WarningCode(String description) {
        this.description = description;
    }

    /**
     * Gets the warning description.
     *
     * @return human-readable warning message
     */
    public String getDescription() {
        return description;
    }

    /**
     * Gets the warning category.
     *
     * @return warning category name
     */
    public String getCategory() {
        String name = name();
        if (name.startsWith("FONT_") || name.contains("FONT")) return "Font";
        if (name.startsWith("COLOR_") || name.contains("COLOR") || name.contains("RGB_") || name.contains("DEVICE_DEPENDENT_")) return "Color";
        if (name.contains("CONTENT") || name.contains("LINK") || name.contains("SCRIPT_") || name.contains("FORM_") || name.contains("ANNOTATION")) return "Content";
        if (name.contains("METADATA") || name.contains("DOCUMENT_INFO")) return "Metadata";
        if (name.contains("IMAGE")) return "Image";
        if (name.contains("STRUCTURE")) return "Structure";
        if (name.contains("TRANSPARENCY") || name.contains("BLEND_")) return "Transparency";
        if (name.contains("ATTACHMENT")) return "Attachment";
        if (name.contains("PRESERVATION")) return "Preservation";
        return "Other";
    }

    /**
     * Gets the severity level of this warning (1-5, with 5 being most severe).
     *
     * @return severity level
     */
    public int getSeverity() {
        switch (this) {
            // High severity warnings
            case MISSING_XMP_METADATA:
            case INCOMPLETE_STRUCTURE_TREE:
            case INCOMPLETE_DOCUMENT_INFO:
                return 4;

            // Medium-high severity
            case UNCALIBRATED_COLOR_SPACE:
            case RGB_COLOR_SPACE_USED:
            case EXTERNAL_LINK_USED:
            case LOW_RESOLUTION_IMAGE:
                return 3;

            // Medium severity
            case FONT_SUBSTITUTION_POSSIBLE:
            case DEVICE_DEPENDENT_COLOR:
            case FORM_FIELD_WITHOUT_EXPORT_VALUE:
            case ANNOTATION_WITHOUT_DATE:
            case LIGHT_TRANSPARENCY_USED:
                return 2;

            // Low severity
            default:
                return 1;
        }
    }

    /**
     * Checks if this warning indicates a potential data loss issue.
     *
     * @return true if warning may cause data loss
     */
    public boolean mayIndicateDataLoss() {
        switch (this) {
            case FONT_SUBSTITUTION_POSSIBLE:
            case UNCALIBRATED_COLOR_SPACE:
            case EXTERNAL_LINK_USED:
            case DEVICE_DEPENDENT_COLOR:
            case INCOMPLETE_STRUCTURE_TREE:
            case LOW_RESOLUTION_IMAGE:
            case MODERN_PDF_FEATURE_USED:
            case OPTIONAL_CONTENT_USED:
                return true;
            default:
                return false;
        }
    }

    /**
     * Parses a warning code from a string.
     *
     * @param codeName warning code name
     * @return corresponding WarningCode
     * @throws IllegalArgumentException if code is not valid
     */
    public static WarningCode parse(String codeName) {
        try {
            return WarningCode.valueOf(codeName.toUpperCase());
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("Invalid warning code: " + codeName, e);
        }
    }
}
