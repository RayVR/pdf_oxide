package com.pdfoxide.compliance;

import java.util.Objects;
import java.util.Optional;

/**
 * Represents a PDF/A compliance warning detected during validation.
 *
 * <p>Warnings indicate potential issues with PDF/A compliance that do not
 * prevent validation success but may affect interoperability or long-term
 * preservation.
 *
 * @since 1.0.0
 */
public final class ComplianceWarning {
    private final WarningCode code;
    private final String message;
    private final Optional<Integer> page;
    private final Optional<String> location;

    /**
     * Constructs a compliance warning.
     *
     * @param code warning code
     * @param message human-readable warning message
     * @param page optional page number where warning occurred (0-based)
     * @param location optional detailed location information
     */
    public ComplianceWarning(
            WarningCode code,
            String message,
            Optional<Integer> page,
            Optional<String> location) {
        this.code = Objects.requireNonNull(code, "code cannot be null");
        this.message = Objects.requireNonNull(message, "message cannot be null");
        this.page = Objects.requireNonNull(page, "page cannot be null");
        this.location = Objects.requireNonNull(location, "location cannot be null");
    }

    /**
     * Gets the warning code.
     *
     * @return WarningCode enum value
     */
    public WarningCode getCode() {
        return code;
    }

    /**
     * Gets the warning message.
     *
     * @return human-readable message
     */
    public String getMessage() {
        return message;
    }

    /**
     * Gets the page where the warning occurred.
     *
     * @return Optional page index (0-based), empty if document-level warning
     */
    public Optional<Integer> getPage() {
        return page;
    }

    /**
     * Gets detailed location information about the warning.
     *
     * @return Optional location string (e.g., "Image ID 5", "Font: Times")
     */
    public Optional<String> getLocation() {
        return location;
    }

    /**
     * Gets the warning category.
     *
     * @return category name
     */
    public String getCategory() {
        return code.getCategory();
    }

    /**
     * Gets the severity level (1-5, with 5 being most severe).
     *
     * @return severity level
     */
    public int getSeverity() {
        return code.getSeverity();
    }

    /**
     * Checks if this warning may indicate data loss.
     *
     * @return true if warning may cause data loss during conversion
     */
    public boolean mayIndicateDataLoss() {
        return code.mayIndicateDataLoss();
    }

    /**
     * Gets a fully formatted warning report.
     *
     * @return formatted string with all available information
     */
    public String getFullReport() {
        StringBuilder sb = new StringBuilder();
        sb.append("[").append(code.name()).append(" - severity ").append(getSeverity()).append("] ");
        sb.append(message);

        if (page.isPresent()) {
            sb.append(" (page ").append(page.get()).append(")");
        }

        if (location.isPresent()) {
            sb.append(" at ").append(location.get());
        }

        if (mayIndicateDataLoss()) {
            sb.append(" ⚠ DATA LOSS RISK");
        }

        return sb.toString();
    }

    @Override
    public String toString() {
        return String.format(
            "ComplianceWarning(code=%s, message='%s', page=%s, severity=%d)",
            code,
            message,
            page.map(p -> "page " + p).orElse("document"),
            getSeverity()
        );
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ComplianceWarning)) return false;
        ComplianceWarning that = (ComplianceWarning) o;
        return code == that.code
                && message.equals(that.message)
                && page.equals(that.page)
                && location.equals(that.location);
    }

    @Override
    public int hashCode() {
        return Objects.hash(code, message, page, location);
    }
}
