package com.pdfoxide.compliance;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

/**
 * Result of a PDF/A compliance validation.
 *
 * <p>Contains validation status, errors, warnings, and statistics for a
 * PDF document validated against specific PDF/A level and part.
 *
 * <p>Example:
 * <pre>{@code
 * ValidationResult result = validator.validate(document);
 *
 * if (result.isValid()) {
 *     System.out.println("Document is valid PDF/A-1b");
 * } else {
 *     System.out.println("Validation failed with errors:");
 *     for (ComplianceError error : result.getErrors()) {
 *         System.out.println("  - " + error.getMessage());
 *     }
 * }
 *
 * System.out.println("Warnings: " + result.getWarningCount());
 * System.out.println("Validation time: " + result.getStats().getValidationTime() + "ms");
 * }</pre>
 *
 * @since 1.0.0
 */
public final class ValidationResult {
    private final PdfALevel level;
    private final PdfAPart part;
    private final boolean valid;
    private final List<ComplianceError> errors;
    private final List<ComplianceWarning> warnings;
    private final ValidationStats stats;

    /**
     * Constructs a validation result.
     *
     * @param level PDF/A level validated against
     * @param part PDF/A part validated against
     * @param valid whether document is valid PDF/A
     * @param errors list of compliance errors (empty if valid)
     * @param warnings list of compliance warnings
     * @param stats validation statistics
     */
    public ValidationResult(
            PdfALevel level,
            PdfAPart part,
            boolean valid,
            List<ComplianceError> errors,
            List<ComplianceWarning> warnings,
            ValidationStats stats) {
        this.level = Objects.requireNonNull(level, "level cannot be null");
        this.part = Objects.requireNonNull(part, "part cannot be null");
        this.valid = valid;
        this.errors = Collections.unmodifiableList(Objects.requireNonNull(errors, "errors cannot be null"));
        this.warnings = Collections.unmodifiableList(Objects.requireNonNull(warnings, "warnings cannot be null"));
        this.stats = Objects.requireNonNull(stats, "stats cannot be null");
    }

    /**
     * Gets the PDF/A level validated against.
     *
     * @return PdfALevel enum value
     */
    public PdfALevel getLevel() {
        return level;
    }

    /**
     * Gets the PDF/A part validated against.
     *
     * @return PdfAPart enum value
     */
    public PdfAPart getPart() {
        return part;
    }

    /**
     * Checks if document is valid PDF/A.
     *
     * @return true if valid, false if errors exist
     */
    public boolean isValid() {
        return valid;
    }

    /**
     * Gets all validation errors.
     *
     * @return unmodifiable list of errors (empty if none)
     */
    public List<ComplianceError> getErrors() {
        return errors;
    }

    /**
     * Gets all validation warnings.
     *
     * @return unmodifiable list of warnings (empty if none)
     */
    public List<ComplianceWarning> getWarnings() {
        return warnings;
    }

    /**
     * Gets validation statistics.
     *
     * @return statistics object with timing and element counts
     */
    public ValidationStats getStats() {
        return stats;
    }

    /**
     * Gets the number of errors.
     *
     * @return error count
     */
    public int getErrorCount() {
        return errors.size();
    }

    /**
     * Gets the number of warnings.
     *
     * @return warning count
     */
    public int getWarningCount() {
        return warnings.size();
    }

    /**
     * Gets errors in a specific category.
     *
     * @param category category name
     * @return filtered error list
     */
    public List<ComplianceError> getErrorsByCategory(String category) {
        return errors.stream()
            .filter(e -> category.equals(e.getCategory()))
            .collect(Collectors.toList());
    }

    /**
     * Gets warnings in a specific category.
     *
     * @param category category name
     * @return filtered warning list
     */
    public List<ComplianceWarning> getWarningsByCategory(String category) {
        return warnings.stream()
            .filter(w -> category.equals(w.getCategory()))
            .collect(Collectors.toList());
    }

    /**
     * Gets warnings with data loss risk.
     *
     * @return list of risky warnings
     */
    public List<ComplianceWarning> getDataLossRiskWarnings() {
        return warnings.stream()
            .filter(ComplianceWarning::mayIndicateDataLoss)
            .collect(Collectors.toList());
    }

    /**
     * Gets high-severity warnings.
     *
     * @return warnings with severity >= 4
     */
    public List<ComplianceWarning> getHighSeverityWarnings() {
        return warnings.stream()
            .filter(w -> w.getSeverity() >= 4)
            .collect(Collectors.toList());
    }

    /**
     * Gets a summary report of validation results.
     *
     * @return formatted summary string
     */
    public String getSummaryReport() {
        StringBuilder sb = new StringBuilder();
        sb.append("PDF/A-").append(level.getCode()).append(" Validation Result\n");
        sb.append("Status: ").append(valid ? "VALID" : "INVALID").append("\n");
        sb.append("Errors: ").append(getErrorCount()).append("\n");
        sb.append("Warnings: ").append(getWarningCount()).append("\n");
        sb.append("High-risk warnings: ").append(getDataLossRiskWarnings().size()).append("\n");
        sb.append("Validation time: ").append(stats.getValidationTime()).append("ms\n");
        sb.append("Pages checked: ").append(stats.getPagesChecked()).append("\n");

        return sb.toString();
    }

    /**
     * Gets a detailed report including all errors and warnings.
     *
     * @return formatted detailed report
     */
    public String getDetailedReport() {
        StringBuilder sb = new StringBuilder();
        sb.append(getSummaryReport());

        if (!errors.isEmpty()) {
            sb.append("\nErrors:\n");
            for (ComplianceError error : errors) {
                sb.append("  ").append(error.getFullReport()).append("\n");
            }
        }

        if (!warnings.isEmpty()) {
            sb.append("\nWarnings:\n");
            for (ComplianceWarning warning : warnings) {
                sb.append("  ").append(warning.getFullReport()).append("\n");
            }
        }

        return sb.toString();
    }

    @Override
    public String toString() {
        return String.format(
            "ValidationResult(level=%s, valid=%s, errors=%d, warnings=%d)",
            level.getCode(),
            valid,
            getErrorCount(),
            getWarningCount()
        );
    }
}
