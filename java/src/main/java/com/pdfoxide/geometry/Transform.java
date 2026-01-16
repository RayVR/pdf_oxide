package com.pdfoxide.geometry;

/**
 * Affine transformation matrix.
 */
public final class Transform {
    private final double a;
    private final double b;
    private final double c;
    private final double d;
    private final double e;
    private final double f;

    public Transform(double a, double b, double c, double d, double e, double f) {
        this.a = a;
        this.b = b;
        this.c = c;
        this.d = d;
        this.e = e;
        this.f = f;
    }

    /**
     * Creates an identity transform.
     */
    public static Transform identity() {
        return new Transform(1, 0, 0, 1, 0, 0);
    }

    public double getA() { return a; }
    public double getB() { return b; }
    public double getC() { return c; }
    public double getD() { return d; }
    public double getE() { return e; }
    public double getF() { return f; }
}
