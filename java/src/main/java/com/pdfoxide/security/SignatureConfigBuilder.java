package com.pdfoxide.security;

/**
 * Builder for signature configuration.
 */
public final class SignatureConfigBuilder {
    private byte[] certificate;
    private byte[] privateKey;
    private String reason;
    private String location;
    private String contactInfo;

    public SignatureConfigBuilder certificate(byte[] cert) {
        this.certificate = cert;
        return this;
    }

    public SignatureConfigBuilder privateKey(byte[] key) {
        this.privateKey = key;
        return this;
    }

    public SignatureConfigBuilder reason(String reason) {
        this.reason = reason;
        return this;
    }

    public SignatureConfigBuilder location(String location) {
        this.location = location;
        return this;
    }

    public SignatureConfigBuilder contactInfo(String info) {
        this.contactInfo = info;
        return this;
    }

    public SignatureConfig build() {
        return new SignatureConfig(certificate, privateKey, reason, location, contactInfo);
    }
}
