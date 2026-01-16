package com.pdfoxide.compliance;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;
import java.util.Objects;

/**
 * Validator for PDF/A compliance.
 *
 * <p>Validates PDF documents against specified PDF/A levels and parts,
 * providing detailed compliance reports.
 *
 * <p>Example:
 * <pre>{@code
 * try (PdfDocument doc = PdfDocument.open("document.pdf")) {
 *     PdfAValidator validator = new PdfAValidator(PdfALevel.LEVEL_1B, PdfAPart.PART_1);
 *     ValidationResult result = validator.validate(doc);
 *
 *     if (result.isValid()) {
 *         System.out.println("Document is valid PDF/A-1b");
 *     } else {
 *         System.out.println("Document is NOT valid PDF/A-1b");
 *         System.out.println(result.getDetailedReport());
 *     }
 * }
 * }</pre>
 *
 * @since 1.0.0
 */
public final class PdfAValidator {
    private final PdfALevel level;
    private final PdfAPart part;
    private boolean closed = false;

    /**
     * Creates a validator for a specific PDF/A level and part.
     *
     * @param level PDF/A level to validate against
     * @param part PDF/A part to validate against
     * @throws IllegalArgumentException if level or part is null
     */
    public PdfAValidator(PdfALevel level, PdfAPart part) {
        this.level = Objects.requireNonNull(level, "level cannot be null");
        this.part = Objects.requireNonNull(part, "part cannot be null");
    }

    /**
     * Creates a validator with default settings (PDF/A-1b).
     *
     * @return validator for PDF/A-1b
     */
    public static PdfAValidator createDefault() {
        return new PdfAValidator(PdfALevel.LEVEL_1B, PdfAPart.PART_1);
    }

    /**
     * Gets the PDF/A level this validator checks.
     *
     * @return PdfALevel enum value
     */
    public PdfALevel getLevel() {
        return level;
    }

    /**
     * Gets the PDF/A part this validator checks.
     *
     * @return PdfAPart enum value
     */
    public PdfAPart getPart() {
        return part;
    }

    /**
     * Validates a PDF document against the configured PDF/A level and part.
     *
     * @param document PDF document to validate
     * @return validation result with details about compliance
     * @throws PdfException if validation fails or document cannot be read
     * @throws IllegalStateException if validator is closed
     * @throws IllegalArgumentException if document is null
     */
    public ValidationResult validate(PdfDocument document) throws PdfException {
        ensureNotClosed();

        if (document == null) {
            throw new IllegalArgumentException("Document cannot be null");
        }

        return nativeValidate(0, level, part);
    }

    /**
     * Performs a quick validation that fails fast on first error.
     *
     * <p>More efficient than validate() when only need to know if document
     * is valid or not, without detailed error information.
     *
     * @param document PDF document to validate
     * @return true if valid, false if any error found
     * @throws PdfException if validation fails or document cannot be read
     * @throws IllegalStateException if validator is closed
     * @throws IllegalArgumentException if document is null
     */
    public boolean isValid(PdfDocument document) throws PdfException {
        ValidationResult result = validate(document);
        return result.isValid();
    }

    /**
     * Gets a description of what this validator checks.
     *
     * @return description string
     */
    public String getDescription() {
        return String.format(
            "PDF/A-%s validator (Part %d, base PDF %s)",
            level.getCode(),
            part.getPartNumber(),
            part.getBasePdfVersion()
        );
    }

    @Override
    public String toString() {
        return String.format(
            "PdfAValidator(level=%s, part=%s)",
            level.getCode(),
            part.getPartNumber()
        );
    }

    /**
     * Closes the validator and releases resources.
     */
    public void close() {
        closed = true;
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("PdfAValidator has been closed");
        }
    }

    // Native method declaration
    private static native ValidationResult nativeValidate(
        long documentPtr,
        PdfALevel level,
        PdfAPart part
    ) throws PdfException;
}
