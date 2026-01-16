package com.pdfoxide.creation;

/**
 * Standard page sizes.
 */
public enum PageSize {
    A0(2384, 3370),
    A1(1684, 2384),
    A2(1191, 1684),
    A3(842, 1191),
    A4(595, 842),
    A5(420, 595),
    A6(298, 420),
    LETTER(612, 792),
    LEGAL(612, 1008),
    TABLOID(792, 1224),
    LEDGER(1224, 792);

    private final double width;
    private final double height;

    PageSize(double width, double height) {
        this.width = width;
        this.height = height;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }
}
