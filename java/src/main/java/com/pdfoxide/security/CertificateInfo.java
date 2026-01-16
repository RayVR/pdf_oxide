package com.pdfoxide.security;

import java.time.Instant;
import java.util.Objects;
import java.util.Optional;

/**
 * Information about an X.509 certificate used in digital signatures.
 *
 * @since 1.0.0
 */
public final class CertificateInfo {
    private final String subject;
    private final String issuer;
    private final String serialNumber;
    private final Instant notBefore;
    private final Instant notAfter;
    private final String thumbprint;
    private final String publicKeyAlgorithm;
    private final int publicKeySize;

    /**
     * Constructs certificate information.
     *
     * @param subject certificate subject Distinguished Name
     * @param issuer certificate issuer Distinguished Name
     * @param serialNumber certificate serial number
     * @param notBefore certificate validity start
     * @param notAfter certificate validity end
     * @param thumbprint certificate SHA-1 thumbprint
     * @param publicKeyAlgorithm public key algorithm (e.g., "RSA", "ECDSA")
     * @param publicKeySize public key size in bits
     */
    public CertificateInfo(
            String subject,
            String issuer,
            String serialNumber,
            Instant notBefore,
            Instant notAfter,
            String thumbprint,
            String publicKeyAlgorithm,
            int publicKeySize) {
        this.subject = Objects.requireNonNull(subject, "subject cannot be null");
        this.issuer = Objects.requireNonNull(issuer, "issuer cannot be null");
        this.serialNumber = Objects.requireNonNull(serialNumber, "serialNumber cannot be null");
        this.notBefore = Objects.requireNonNull(notBefore, "notBefore cannot be null");
        this.notAfter = Objects.requireNonNull(notAfter, "notAfter cannot be null");
        this.thumbprint = Objects.requireNonNull(thumbprint, "thumbprint cannot be null");
        this.publicKeyAlgorithm = Objects.requireNonNull(publicKeyAlgorithm, "publicKeyAlgorithm cannot be null");
        this.publicKeySize = publicKeySize;
    }

    /**
     * Gets the certificate subject Distinguished Name.
     *
     * @return subject DN string
     */
    public String getSubject() {
        return subject;
    }

    /**
     * Gets the certificate issuer Distinguished Name.
     *
     * @return issuer DN string
     */
    public String getIssuer() {
        return issuer;
    }

    /**
     * Gets the certificate serial number.
     *
     * @return serial number string
     */
    public String getSerialNumber() {
        return serialNumber;
    }

    /**
     * Gets the certificate validity start time.
     *
     * @return notBefore instant
     */
    public Instant getNotBefore() {
        return notBefore;
    }

    /**
     * Gets the certificate validity end time.
     *
     * @return notAfter instant
     */
    public Instant getNotAfter() {
        return notAfter;
    }

    /**
     * Gets the certificate SHA-1 thumbprint.
     *
     * @return thumbprint as hex string
     */
    public String getThumbprint() {
        return thumbprint;
    }

    /**
     * Gets the public key algorithm.
     *
     * @return algorithm name (e.g., "RSA", "ECDSA")
     */
    public String getPublicKeyAlgorithm() {
        return publicKeyAlgorithm;
    }

    /**
     * Gets the public key size in bits.
     *
     * @return key size (e.g., 2048, 4096)
     */
    public int getPublicKeySize() {
        return publicKeySize;
    }

    /**
     * Checks if certificate is currently valid.
     *
     * @return true if now is between notBefore and notAfter
     */
    public boolean isValid() {
        Instant now = Instant.now();
        return !now.isBefore(notBefore) && !now.isAfter(notAfter);
    }

    /**
     * Checks if certificate has expired.
     *
     * @return true if now is after notAfter
     */
    public boolean isExpired() {
        return Instant.now().isAfter(notAfter);
    }

    /**
     * Checks if certificate is not yet valid.
     *
     * @return true if now is before notBefore
     */
    public boolean isNotYetValid() {
        return Instant.now().isBefore(notBefore);
    }

    /**
     * Gets remaining validity in days.
     *
     * @return days until expiration (negative if expired)
     */
    public long getRemainingValidityDays() {
        long remainingMillis = notAfter.toEpochMilli() - System.currentTimeMillis();
        return remainingMillis / (24 * 60 * 60 * 1000);
    }

    /**
     * Gets a formatted certificate info string.
     *
     * @return formatted information
     */
    public String getFormattedInfo() {
        StringBuilder sb = new StringBuilder();
        sb.append("Subject: ").append(subject).append("\n");
        sb.append("Issuer: ").append(issuer).append("\n");
        sb.append("Serial: ").append(serialNumber).append("\n");
        sb.append("Valid: ").append(notBefore).append(" to ").append(notAfter).append("\n");
        sb.append("Status: ");

        if (isExpired()) {
            sb.append("EXPIRED");
        } else if (isNotYetValid()) {
            sb.append("NOT YET VALID");
        } else {
            sb.append("VALID (").append(getRemainingValidityDays()).append(" days remaining)");
        }
        sb.append("\n");

        sb.append("Algorithm: ").append(publicKeyAlgorithm);
        sb.append(" (").append(publicKeySize).append("-bit)").append("\n");
        sb.append("Thumbprint: ").append(thumbprint);

        return sb.toString();
    }

    @Override
    public String toString() {
        return String.format(
            "CertificateInfo(subject='%s', serial=%s, valid=%s, expires=%s)",
            subject,
            serialNumber,
            isValid(),
            notAfter
        );
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (!(o instanceof CertificateInfo)) return false;
        CertificateInfo that = (CertificateInfo) o;
        return publicKeySize == that.publicKeySize
                && subject.equals(that.subject)
                && serialNumber.equals(that.serialNumber);
    }

    @Override
    public int hashCode() {
        return Objects.hash(subject, serialNumber, publicKeySize);
    }
}
