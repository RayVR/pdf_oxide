package com.pdfoxide.compliance;

import java.util.Objects;
import java.util.Optional;

/**
 * Represents a PDF/A compliance error detected during validation.
 *
 * <p>Errors indicate critical non-compliance with PDF/A specifications that
 * must be fixed for the document to be valid PDF/A.
 *
 * @since 1.0.0
 */
public final class ComplianceError {
    private final ErrorCode code;
    private final String message;
    private final Optional<Integer> page;
    private final Optional<String> location;

    /**
     * Constructs a compliance error.
     *
     * @param code error code
     * @param message human-readable error message
     * @param page optional page number where error occurred (0-based)
     * @param location optional detailed location information
     */
    public ComplianceError(
            ErrorCode code,
            String message,
            Optional<Integer> page,
            Optional<String> location) {
        this.code = Objects.requireNonNull(code, "code cannot be null");
        this.message = Objects.requireNonNull(message, "message cannot be null");
        this.page = Objects.requireNonNull(page, "page cannot be null");
        this.location = Objects.requireNonNull(location, "location cannot be null");
    }

    /**
     * Gets the error code.
     *
     * @return ErrorCode enum value
     */
    public ErrorCode getCode() {
        return code;
    }

    /**
     * Gets the error message.
     *
     * @return human-readable message
     */
    public String getMessage() {
        return message;
    }

    /**
     * Gets the page where the error occurred.
     *
     * @return Optional page index (0-based), empty if document-level error
     */
    public Optional<Integer> getPage() {
        return page;
    }

    /**
     * Gets detailed location information about the error.
     *
     * @return Optional location string (e.g., "Image ID 42", "Font: Helvetica")
     */
    public Optional<String> getLocation() {
        return location;
    }

    /**
     * Gets the error category.
     *
     * @return category name
     */
    public String getCategory() {
        return code.getCategory();
    }

    /**
     * Gets a fully formatted error report.
     *
     * @return formatted string with all available information
     */
    public String getFullReport() {
        StringBuilder sb = new StringBuilder();
        sb.append("[").append(code.name()).append("] ");
        sb.append(message);

        if (page.isPresent()) {
            sb.append(" (page ").append(page.get()).append(")");
        }

        if (location.isPresent()) {
            sb.append(" at ").append(location.get());
        }

        return sb.toString();
    }

    @Override
    public String toString() {
        return String.format(
            "ComplianceError(code=%s, message='%s', page=%s, location=%s)",
            code,
            message,
            page.map(Object::toString).orElse("(all)"),
            location.orElse("(unspecified)")
        );
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof ComplianceError)) return false;
        ComplianceError that = (ComplianceError) o;
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
