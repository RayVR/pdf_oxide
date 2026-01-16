package com.pdfoxide.compliance;

/**
 * PDF/A validation error codes.
 *
 * <p>Enumeration of error conditions that indicate non-compliance with PDF/A
 * specifications. Errors are critical issues that prevent a document from being
 * valid PDF/A.
 *
 * @since 1.0.0
 */
public enum ErrorCode {
    // Encryption and Security errors
    ENCRYPTION_NOT_ALLOWED("Encryption is not allowed in PDF/A"),
    INVALID_ENCRYPTION_ALGORITHM("Invalid or non-compliant encryption algorithm"),

    // Font errors
    MISSING_REQUIRED_FONT("Required font is missing"),
    INVALID_FONT_ENCODING("Font encoding is invalid for PDF/A"),
    EMBEDDED_FONT_REQUIRED("Font must be embedded"),
    INVALID_FONT_PROGRAM("Font program is invalid or corrupted"),

    // Color and Graphics errors
    INVALID_COLOR_SPACE("Invalid color space for PDF/A"),
    CMYK_COLOR_SPACE_REQUIRED("CMYK color space required but not used"),
    MISSING_ICC_PROFILE("ICC color profile is required but missing"),
    INVALID_RENDERING_INTENT("Invalid rendering intent specified"),
    EXTERNAL_XMP_REFERENCE("External XMP reference not allowed"),

    // Content and Structure errors
    EXTERNAL_CONTENT_REFERENCE("Reference to external content not allowed"),
    JAVASCRIPT_NOT_ALLOWED("JavaScript is not allowed in PDF/A"),
    LAUNCH_ACTIONS_NOT_ALLOWED("Launch actions are not allowed"),
    FORM_XFA_NOT_ALLOWED("XFA forms are not allowed in PDF/A"),
    MISSING_LOGICAL_STRUCTURE("Logical structure is required but missing (PDF/A-1a, 2a, 3a)"),
    INVALID_STRUCTURE_TREE("Structure tree is invalid or incomplete"),
    MISSING_ALTERNATE_DESCRIPTIONS("Alternate descriptions are required but missing"),
    INVALID_TABLE_STRUCTURE("Table structure is invalid"),

    // Metadata errors
    MISSING_DOCUMENT_INFO("Document information dictionary is missing"),
    INVALID_DOCUMENT_METADATA("Document metadata is invalid"),
    MISSING_MODIFICATION_DATE("Modification date is required but missing"),
    INVALID_CREATION_DATE("Invalid creation date format"),

    // Annotation errors
    INVALID_ANNOTATION_TYPE("Annotation type not allowed in PDF/A"),
    ANNOTATION_WITHOUT_APPEARANCE("Annotation without appearance stream"),
    POPUP_ANNOTATION_WITHOUT_PARENT("Popup annotation without parent"),

    // Transparency errors
    TRANSPARENCY_NOT_ALLOWED("Transparency is not allowed"),
    SOFT_MASK_NOT_ALLOWED("Soft masks are not allowed"),
    INVALID_BLEND_MODE("Invalid blend mode for PDF/A"),

    // 3D and Multimedia errors
    MULTIMEDIA_NOT_ALLOWED("Multimedia content not allowed"),
    THREE_D_CONTENT_NOT_ALLOWED("3D content not allowed in this PDF/A level"),

    // File attachment errors
    FILE_ATTACHMENT_NOT_ALLOWED("File attachments not allowed in this PDF/A level"),

    // Other compliance errors
    INVALID_DOCUMENT_STRUCTURE("Invalid document structure"),
    MISSING_REQUIRED_ENTRY("Required dictionary entry is missing"),
    RESERVED_OPERATOR_USED("Reserved operator used in content stream"),
    INVALID_PAGE_SIZE("Invalid or missing page size"),
    UNRECOGNIZED_FILTER("Unrecognized or non-compliant filter"),
    VALIDATION_FAILED("General validation failure");

    private final String description;

    ErrorCode(String description) {
        this.description = description;
    }

    /**
     * Gets the error description.
     *
     * @return human-readable error message
     */
    public String getDescription() {
        return description;
    }

    /**
     * Gets the error category.
     *
     * @return error category name
     */
    public String getCategory() {
        String name = name();
        if (name.startsWith("ENCRYPTION_")) return "Encryption";
        if (name.startsWith("FONT_") || name.contains("FONT")) return "Font";
        if (name.startsWith("COLOR_") || name.startsWith("CMYK_") || name.startsWith("ICC_") || name.startsWith("INVALID_RENDERING_")) return "Color";
        if (name.contains("CONTENT") || name.contains("JAVASCRIPT") || name.contains("LAUNCH_") || name.contains("XFA") || name.contains("STRUCTURE")) return "Content";
        if (name.contains("METADATA") || name.contains("DATE")) return "Metadata";
        if (name.contains("ANNOTATION")) return "Annotation";
        if (name.contains("TRANSPARENCY") || name.contains("BLEND_") || name.contains("MASK")) return "Transparency";
        if (name.contains("MULTIMEDIA") || name.contains("THREE_D")) return "Multimedia";
        if (name.contains("ATTACHMENT")) return "Attachment";
        return "Other";
    }

    /**
     * Checks if this error indicates a critical compliance issue.
     *
     * @return true for all errors (all errors are critical)
     */
    public boolean isCritical() {
        return true;
    }

    /**
     * Parses an error code from a string.
     *
     * @param codeName error code name
     * @return corresponding ErrorCode
     * @throws IllegalArgumentException if code is not valid
     */
    public static ErrorCode parse(String codeName) {
        try {
            return ErrorCode.valueOf(codeName.toUpperCase());
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("Invalid error code: " + codeName, e);
        }
    }
}
