package com.pdfoxide.annotations;

/**
 * Actions for link annotations.
 */
public final class LinkAction {
    private final String type;
    private final String target;

    private LinkAction(String type, String target) {
        this.type = type;
        this.target = target;
    }

    /**
     * Creates external link action.
     *
     * @param url target URL
     * @return link action
     */
    public static LinkAction externalLink(String url) {
        return new LinkAction("url", url);
    }

    /**
     * Creates internal link action.
     *
     * @param pageIndex target page index
     * @return link action
     */
    public static LinkAction internalLink(int pageIndex) {
        return new LinkAction("page", String.valueOf(pageIndex));
    }

    public String getType() {
        return type;
    }

    public String getTarget() {
        return target;
    }
}
