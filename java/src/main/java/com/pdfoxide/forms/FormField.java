package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Optional;

/**
 * Base interface for PDF form fields.
 *
 * @since 1.0.0
 */
public interface FormField {
    /**
     * Gets the field name.
     *
     * @return field name
     */
    String getName();

    /**
     * Gets the field type.
     *
     * @return field type
     */
    FormFieldType getFieldType();

    /**
     * Gets the field value.
     *
     * @return current value
     */
    FormFieldValue getValue();

    /**
     * Gets the default value.
     *
     * @return default value, empty if none set
     */
    Optional<FormFieldValue> getDefaultValue();

    /**
     * Gets the field's widget area on page.
     *
     * @return bounding rectangle
     */
    Rect getRect();

    /**
     * Gets the field tooltip.
     *
     * @return tooltip text, empty if none
     */
    Optional<String> getTooltip();

    /**
     * Checks if field is read-only.
     *
     * @return true if read-only
     */
    boolean isReadOnly();

    /**
     * Checks if field is required.
     *
     * @return true if required
     */
    boolean isRequired();

    /**
     * Checks if field is hidden.
     *
     * @return true if hidden
     */
    boolean isHidden();

    /**
     * Checks if field is disabled.
     *
     * @return true if disabled
     */
    boolean isDisabled();
}
