package com.pdfoxide.forms;

import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * Form field value - sealed interface for type-safe field values.
 *
 * <p>Represents different value types that form fields can hold: text,
 * boolean, named values, or arrays of values.
 *
 * @since 1.0.0
 */
public sealed interface FormFieldValue permits
    FormFieldValue.TextValue,
    FormFieldValue.BooleanValue,
    FormFieldValue.NameValue,
    FormFieldValue.ArrayValue,
    FormFieldValue.NullValue {

    /**
     * Text field value.
     */
    final class TextValue implements FormFieldValue {
        private final String value;

        public TextValue(String value) {
            this.value = Objects.requireNonNull(value, "Text value cannot be null");
        }

        public String getValue() {
            return value;
        }

        @Override
        public String toString() {
            return String.format("TextValue('%s')", value);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof TextValue)) return false;
            return value.equals(((TextValue) o).value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }
    }

    /**
     * Boolean field value.
     */
    final class BooleanValue implements FormFieldValue {
        private final boolean value;

        public BooleanValue(boolean value) {
            this.value = value;
        }

        public boolean getValue() {
            return value;
        }

        @Override
        public String toString() {
            return String.format("BooleanValue(%s)", value);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof BooleanValue)) return false;
            return value == ((BooleanValue) o).value;
        }

        @Override
        public int hashCode() {
            return Boolean.hashCode(value);
        }
    }

    /**
     * Named value (like radio button or dropdown selection).
     */
    final class NameValue implements FormFieldValue {
        private final String value;

        public NameValue(String value) {
            this.value = Objects.requireNonNull(value, "Name value cannot be null");
        }

        public String getValue() {
            return value;
        }

        @Override
        public String toString() {
            return String.format("NameValue('%s')", value);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof NameValue)) return false;
            return value.equals(((NameValue) o).value);
        }

        @Override
        public int hashCode() {
            return value.hashCode();
        }
    }

    /**
     * Array value (like multi-select list).
     */
    final class ArrayValue implements FormFieldValue {
        private final List<String> values;

        public ArrayValue(String... values) {
            this(Arrays.asList(values));
        }

        public ArrayValue(List<String> values) {
            this.values = List.copyOf(Objects.requireNonNull(values, "Array values cannot be null"));
        }

        public List<String> getValues() {
            return values;
        }

        @Override
        public String toString() {
            return String.format("ArrayValue(%s)", values);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (!(o instanceof ArrayValue)) return false;
            return values.equals(((ArrayValue) o).values);
        }

        @Override
        public int hashCode() {
            return values.hashCode();
        }
    }

    /**
     * Null/empty value.
     */
    final class NullValue implements FormFieldValue {
        public static final NullValue INSTANCE = new NullValue();

        private NullValue() {}

        @Override
        public String toString() {
            return "NullValue()";
        }
    }

    /**
     * Creates a text value.
     *
     * @param text text content
     * @return text value
     */
    static FormFieldValue text(String text) {
        return new TextValue(text);
    }

    /**
     * Creates a boolean value.
     *
     * @param checked checked state
     * @return boolean value
     */
    static FormFieldValue bool(boolean checked) {
        return new BooleanValue(checked);
    }

    /**
     * Creates a named value.
     *
     * @param name named value
     * @return name value
     */
    static FormFieldValue name(String name) {
        return new NameValue(name);
    }

    /**
     * Creates an array value.
     *
     * @param values array items
     * @return array value
     */
    static FormFieldValue array(String... values) {
        return new ArrayValue(values);
    }

    /**
     * Creates an array value from list.
     *
     * @param values list of values
     * @return array value
     */
    static FormFieldValue array(List<String> values) {
        return new ArrayValue(values);
    }

    /**
     * Gets null value.
     *
     * @return null value
     */
    static FormFieldValue NULL() {
        return NullValue.INSTANCE;
    }
}
