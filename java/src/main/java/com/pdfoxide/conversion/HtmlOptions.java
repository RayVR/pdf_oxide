package com.pdfoxide.conversion;

/**
 * Options specific to HTML conversion.
 */
public final class HtmlOptions {
    private final boolean includeStyles;
    private final boolean includeScripts;
    private final boolean makeResponsive;

    public HtmlOptions(boolean includeStyles, boolean includeScripts, boolean makeResponsive) {
        this.includeStyles = includeStyles;
        this.includeScripts = includeScripts;
        this.makeResponsive = makeResponsive;
    }

    public boolean isIncludeStyles() {
        return includeStyles;
    }

    public boolean isIncludeScripts() {
        return includeScripts;
    }

    public boolean isMakeResponsive() {
        return makeResponsive;
    }
}
