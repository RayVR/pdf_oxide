package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for signature fields.
 */
public final class SignatureFieldBuilder {
    private final String name;
    private final Rect rect;
    private String reason;
    private String location;

    private SignatureFieldBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static SignatureFieldBuilder create(String name, Rect rect) {
        return new SignatureFieldBuilder(name, rect);
    }

    public SignatureFieldBuilder reason(String reason) {
        this.reason = reason;
        return this;
    }

    public SignatureFieldBuilder location(String location) {
        this.location = location;
        return this;
    }

    public SignatureField build() {
        return new SignatureField(name, rect, reason, location);
    }
}
