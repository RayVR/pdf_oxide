package com.pdfoxide.dom;

import java.util.UUID;

/**
 * Unique identifier for PDF elements in the DOM tree.
 *
 * <p>ElementIds are used to reference elements for modification and navigation.
 * They are UUID-based and guaranteed to be unique within a page.
 *
 * @since 1.0.0
 */
public final class ElementId {
    private final long mostSigBits;
    private final long leastSigBits;

    /**
     * Creates an ElementId from UUID components.
     *
     * @param mostSigBits most significant bits
     * @param leastSigBits least significant bits
     */
    ElementId(long mostSigBits, long leastSigBits) {
        this.mostSigBits = mostSigBits;
        this.leastSigBits = leastSigBits;
    }

    /**
     * Creates a new random ElementId.
     *
     * @return new unique element ID
     */
    public static ElementId generate() {
        UUID uuid = UUID.randomUUID();
        return new ElementId(uuid.getMostSignificantBits(), uuid.getLeastSignificantBits());
    }

    /**
     * Gets the most significant bits of the UUID.
     *
     * @return most significant bits
     */
    public long getMostSignificantBits() {
        return mostSigBits;
    }

    /**
     * Gets the least significant bits of the UUID.
     *
     * @return least significant bits
     */
    public long getLeastSignificantBits() {
        return leastSigBits;
    }

    /**
     * Converts to UUID representation.
     *
     * @return UUID with same bits
     */
    public UUID toUUID() {
        return new UUID(mostSigBits, leastSigBits);
    }

    @Override
    public boolean equals(Object obj) {
        if (!(obj instanceof ElementId)) {
            return false;
        }
        ElementId other = (ElementId) obj;
        return mostSigBits == other.mostSigBits && leastSigBits == other.leastSigBits;
    }

    @Override
    public int hashCode() {
        return (int) ((mostSigBits ^ leastSigBits) ^ (mostSigBits >>> 32) ^ (leastSigBits >>> 32));
    }

    @Override
    public String toString() {
        return toUUID().toString();
    }
}
