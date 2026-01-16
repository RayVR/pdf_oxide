package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.time.Instant;
import java.util.Optional;

/**
 * Signature field for digital signatures.
 *
 * @since 1.0.0
 */
public final class SignatureField implements FormField {
    private final String name;
    private final Rect rect;
    private final Optional<String> tooltip;
    private final boolean readOnly;
    private final boolean required;
    private final boolean hidden;
    private final boolean disabled;
    private final Optional<String> reason;
    private final Optional<String> location;
    private final Optional<String> contactInfo;
    private final Optional<Instant> signedDate;
    private final boolean isSigned;
    private final Optional<String> signerName;
    private final Optional<String> certificateInfo;

    /**
     * Constructs a signature field.
     *
     * @param name field name
     * @param rect widget rectangle
     * @param tooltip tooltip text (optional)
     * @param readOnly read-only state
     * @param required required state
     * @param hidden hidden state
     * @param disabled disabled state
     * @param reason reason for signing (optional)
     * @param location signing location (optional)
     * @param contactInfo contact information (optional)
     * @param signedDate signature date (optional)
     * @param isSigned whether field has been signed
     * @param signerName name of signer (optional)
     * @param certificateInfo certificate information (optional)
     */
    private SignatureField(
            String name,
            Rect rect,
            Optional<String> tooltip,
            boolean readOnly,
            boolean required,
            boolean hidden,
            boolean disabled,
            Optional<String> reason,
            Optional<String> location,
            Optional<String> contactInfo,
            Optional<Instant> signedDate,
            boolean isSigned,
            Optional<String> signerName,
            Optional<String> certificateInfo) {
        this.name = name;
        this.rect = rect;
        this.tooltip = tooltip;
        this.readOnly = readOnly;
        this.required = required;
        this.hidden = hidden;
        this.disabled = disabled;
        this.reason = reason;
        this.location = location;
        this.contactInfo = contactInfo;
        this.signedDate = signedDate;
        this.isSigned = isSigned;
        this.signerName = signerName;
        this.certificateInfo = certificateInfo;
    }

    @Override
    public String getName() {
        return name;
    }

    @Override
    public FormFieldType getFieldType() {
        return FormFieldType.SIGNATURE;
    }

    @Override
    public FormFieldValue getValue() {
        return isSigned
            ? FormFieldValue.name("Signed")
            : FormFieldValue.NULL();
    }

    @Override
    public Optional<FormFieldValue> getDefaultValue() {
        return Optional.empty();
    }

    @Override
    public Rect getRect() {
        return rect;
    }

    @Override
    public Optional<String> getTooltip() {
        return tooltip;
    }

    @Override
    public boolean isReadOnly() {
        return readOnly;
    }

    @Override
    public boolean isRequired() {
        return required;
    }

    @Override
    public boolean isHidden() {
        return hidden;
    }

    @Override
    public boolean isDisabled() {
        return disabled;
    }

    public Optional<String> getReason() {
        return reason;
    }

    public Optional<String> getLocation() {
        return location;
    }

    public Optional<String> getContactInfo() {
        return contactInfo;
    }

    public Optional<Instant> getSignedDate() {
        return signedDate;
    }

    public boolean isSigned() {
        return isSigned;
    }

    public Optional<String> getSignerName() {
        return signerName;
    }

    public Optional<String> getCertificateInfo() {
        return certificateInfo;
    }

    public static Builder builder(String name, Rect rect) {
        return new Builder(name, rect);
    }

    public static final class Builder {
        private final String name;
        private final Rect rect;
        private Optional<String> tooltip = Optional.empty();
        private boolean readOnly = false;
        private boolean required = true;  // Signature fields typically required
        private boolean hidden = false;
        private boolean disabled = false;
        private Optional<String> reason = Optional.empty();
        private Optional<String> location = Optional.empty();
        private Optional<String> contactInfo = Optional.empty();
        private Optional<Instant> signedDate = Optional.empty();
        private boolean isSigned = false;
        private Optional<String> signerName = Optional.empty();
        private Optional<String> certificateInfo = Optional.empty();

        private Builder(String name, Rect rect) {
            this.name = name;
            this.rect = rect;
        }

        public Builder tooltip(String tooltip) {
            this.tooltip = Optional.of(tooltip);
            return this;
        }

        public Builder readOnly(boolean readOnly) {
            this.readOnly = readOnly;
            return this;
        }

        public Builder required(boolean required) {
            this.required = required;
            return this;
        }

        public Builder hidden(boolean hidden) {
            this.hidden = hidden;
            return this;
        }

        public Builder disabled(boolean disabled) {
            this.disabled = disabled;
            return this;
        }

        public Builder reason(String reason) {
            this.reason = Optional.of(reason);
            return this;
        }

        public Builder location(String location) {
            this.location = Optional.of(location);
            return this;
        }

        public Builder contactInfo(String contact) {
            this.contactInfo = Optional.of(contact);
            return this;
        }

        public Builder signedDate(Instant date) {
            this.signedDate = Optional.of(date);
            return this;
        }

        public Builder signed(boolean signed) {
            this.isSigned = signed;
            return this;
        }

        public Builder signerName(String name) {
            this.signerName = Optional.of(name);
            return this;
        }

        public Builder certificateInfo(String info) {
            this.certificateInfo = Optional.of(info);
            return this;
        }

        public SignatureField build() {
            return new SignatureField(
                name,
                rect,
                tooltip,
                readOnly,
                required,
                hidden,
                disabled,
                reason,
                location,
                contactInfo,
                signedDate,
                isSigned,
                signerName,
                certificateInfo
            );
        }
    }

    @Override
    public String toString() {
        return String.format(
            "SignatureField(name='%s', signed=%s, signer=%s, rect=%s)",
            name,
            isSigned,
            signerName.orElse("(unsigned)"),
            rect
        );
    }
}
