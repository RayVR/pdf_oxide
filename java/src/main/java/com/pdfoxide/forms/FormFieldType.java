package com.pdfoxide.forms;

/**
 * Form field types according to PDF specification.
 *
 * @since 1.0.0
 */
public enum FormFieldType {
    /**
     * Text field - single or multi-line text input.
     */
    TEXT("Tx"),

    /**
     * Button field - push button, radio button, or checkbox.
     */
    BUTTON("Btn"),

    /**
     * Choice field - combo box or list box.
     */
    CHOICE("Ch"),

    /**
     * Signature field - digital signature.
     */
    SIGNATURE("Sig");

    private final String pdfType;

    FormFieldType(String pdfType) {
        this.pdfType = pdfType;
    }

    /**
     * Gets the PDF field type abbreviation.
     *
     * @return PDF field type code
     */
    public String getPdfType() {
        return pdfType;
    }

    /**
     * Gets the field type from PDF type code.
     *
     * @param pdfType PDF field type abbreviation
     * @return field type, or TEXT if not recognized
     */
    public static FormFieldType fromPdfType(String pdfType) {
        for (FormFieldType type : values()) {
            if (type.pdfType.equals(pdfType)) {
                return type;
            }
        }
        return TEXT;
    }
}
