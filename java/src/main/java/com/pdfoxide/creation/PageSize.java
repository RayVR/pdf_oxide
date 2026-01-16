package com.pdfoxide.creation;

/**
 * Standard paper sizes for PDF creation.
 */
public enum PageSize {
    LETTER(612, 792),
    LEGAL(612, 1008),
    A0(2384, 3370),
    A1(1684, 2384),
    A2(1191, 1684),
    A3(842, 1191),
    A4(595, 842),
    A5(420, 595),
    A6(298, 420),
    B4(1000, 1414),
    B5(707, 1000),
    B6(500, 707);

    private final int width;
    private final int height;

    PageSize(int width, int height) {
        this.width = width;
        this.height = height;
    }

    public int getWidth() { return width; }
    public int getHeight() { return height; }
}
