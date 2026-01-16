package com.pdfoxide.geometry;

/**
 * 2D transformation matrix.
 */
public final class Matrix {
    private final double[][] values;

    public Matrix(double[][] values) {
        if (values.length != 3 || values[0].length != 3) {
            throw new IllegalArgumentException("Matrix must be 3x3");
        }
        this.values = values;
    }

    public double[][] getValues() {
        return values;
    }

    public static Matrix identity() {
        return new Matrix(new double[][] {
            {1, 0, 0},
            {0, 1, 0},
            {0, 0, 1}
        });
    }
}
