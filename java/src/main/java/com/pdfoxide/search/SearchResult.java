package com.pdfoxide.search;

/**
 * Result of a text search operation.
 */
public final class SearchResult {
    private final String text;
    private final int page;
    private final double x;
    private final double y;
    private final double width;
    private final double height;

    public SearchResult(String text, int page, double x, double y, double width, double height) {
        this.text = text;
        this.page = page;
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }

    public String getText() {
        return text;
    }

    public int getPage() {
        return page;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    @Override
    public String toString() {
        return "SearchResult{" +
                "text='" + text + '\'' +
                ", page=" + page +
                ", x=" + x +
                ", y=" + y +
                ", width=" + width +
                ", height=" + height +
                '}';
    }
}
