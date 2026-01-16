package com.pdfoxide.dom;

import java.util.Optional;

/**
 * Represents a table cell in PDF content.
 */
public final class TableCell {
    private final int row;
    private final int column;
    private final String content;
    private final double x;
    private final double y;
    private final double width;
    private final double height;

    public TableCell(int row, int column, String content, double x, double y, double width, double height) {
        this.row = row;
        this.column = column;
        this.content = content;
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }

    public int getRow() { return row; }
    public int getColumn() { return column; }
    public Optional<String> getContent() { return Optional.ofNullable(content); }
    public double getX() { return x; }
    public double getY() { return y; }
    public double getWidth() { return width; }
    public double getHeight() { return height; }
}
