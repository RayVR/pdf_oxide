package com.pdfoxide.security;

import java.util.Objects;
import java.util.Optional;

/**
 * Configuration for digital signature operations.
 *
 * <p>Specifies certificate, key, and metadata for signing PDF documents.
 *
 * @since 1.0.0
 */
public final class SignatureConfig {
    private final byte[] certificate;
    private final byte[] privateKey;
    private final Optional<String> reason;
    private final Optional<String> location;
    private final Optional<String> contactInfo;
    private final Optional<String> password;

    /**
     * Constructs signature configuration.
     *
     * @param certificate X.509 certificate in PEM or DER format
     * @param privateKey private key in PEM or DER format
     * @param reason optional reason for signing
     * @param location optional location of signing
     * @param contactInfo optional contact information
     * @param password optional password for private key
     */
    private SignatureConfig(
            byte[] certificate,
            byte[] privateKey,
            Optional<String> reason,
            Optional<String> location,
            Optional<String> contactInfo,
            Optional<String> password) {
        this.certificate = Objects.requireNonNull(certificate, "certificate cannot be null");
        this.privateKey = Objects.requireNonNull(privateKey, "privateKey cannot be null");
        this.reason = Objects.requireNonNull(reason, "reason cannot be null");
        this.location = Objects.requireNonNull(location, "location cannot be null");
        this.contactInfo = Objects.requireNonNull(contactInfo, "contactInfo cannot be null");
        this.password = Objects.requireNonNull(password, "password cannot be null");
    }

    /**
     * Creates a builder for signature configuration.
     *
     * @return new builder
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * Gets the certificate.
     *
     * @return X.509 certificate bytes
     */
    public byte[] getCertificate() {
        return certificate.clone();
    }

    /**
     * Gets the private key.
     *
     * @return private key bytes
     */
    public byte[] getPrivateKey() {
        return privateKey.clone();
    }

    /**
     * Gets the reason for signing.
     *
     * @return Optional reason string
     */
    public Optional<String> getReason() {
        return reason;
    }

    /**
     * Gets the location of signing.
     *
     * @return Optional location string
     */
    public Optional<String> getLocation() {
        return location;
    }

    /**
     * Gets the contact information.
     *
     * @return Optional contact string
     */
    public Optional<String> getContactInfo() {
        return contactInfo;
    }

    /**
     * Gets the private key password.
     *
     * @return Optional password
     */
    public Optional<String> getPassword() {
        return password;
    }

    /**
     * Builder for fluent SignatureConfig construction.
     */
    public static final class Builder {
        private byte[] certificate;
        private byte[] privateKey;
        private String reason;
        private String location;
        private String contactInfo;
        private String password;

        private Builder() {}

        /**
         * Sets the X.509 certificate.
         *
         * @param certBytes certificate in PEM or DER format
         * @return this builder
         */
        public Builder certificate(byte[] certBytes) {
            this.certificate = Objects.requireNonNull(certBytes, "certificate cannot be null");
            return this;
        }

        /**
         * Sets the private key.
         *
         * @param keyBytes private key in PEM or DER format
         * @return this builder
         */
        public Builder privateKey(byte[] keyBytes) {
            this.privateKey = Objects.requireNonNull(keyBytes, "private key cannot be null");
            return this;
        }

        /**
         * Sets the reason for signing.
         *
         * @param reason reason string
         * @return this builder
         */
        public Builder reason(String reason) {
            this.reason = reason;
            return this;
        }

        /**
         * Sets the location of signing.
         *
         * @param location location string
         * @return this builder
         */
        public Builder location(String location) {
            this.location = location;
            return this;
        }

        /**
         * Sets contact information.
         *
         * @param contactInfo contact string
         * @return this builder
         */
        public Builder contactInfo(String contactInfo) {
            this.contactInfo = contactInfo;
            return this;
        }

        /**
         * Sets password for private key.
         *
         * @param password password string
         * @return this builder
         */
        public Builder password(String password) {
            this.password = password;
            return this;
        }

        /**
         * Builds the SignatureConfig.
         *
         * @return configured SignatureConfig
         * @throws IllegalStateException if certificate or private key is not set
         */
        public SignatureConfig build() {
            if (certificate == null) {
                throw new IllegalStateException("Certificate must be set");
            }
            if (privateKey == null) {
                throw new IllegalStateException("Private key must be set");
            }

            return new SignatureConfig(
                certificate,
                privateKey,
                Optional.ofNullable(reason),
                Optional.ofNullable(location),
                Optional.ofNullable(contactInfo),
                Optional.ofNullable(password)
            );
        }
    }
}
