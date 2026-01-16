#!/bin/bash
# Generate DOM content models and additional supporting classes

BASE_DIR="/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

python3 << 'PYTHON_EOF'
import os

base_dir = "/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

files = {
    # DOM Content models
    "dom/TextContent.java": """package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;

/**
 * Text content model for PDF pages.
 */
public final class TextContent {
    private final String text;
    private final double x;
    private final double y;
    private final double fontSize;
    private final String fontName;
    private final double[] color;

    private TextContent(Builder builder) {
        this.text = builder.text;
        this.x = builder.x;
        this.y = builder.y;
        this.fontSize = builder.fontSize;
        this.fontName = builder.fontName;
        this.color = builder.color;
    }

    public static Builder builder() {
        return new Builder();
    }

    public String getText() {
        return text;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getFontSize() {
        return fontSize;
    }

    public String getFontName() {
        return fontName;
    }

    public double[] getColor() {
        return color;
    }

    public static final class Builder {
        private String text;
        private double x;
        private double y;
        private double fontSize = 12.0;
        private String fontName = "Helvetica";
        private double[] color = new double[]{0.0, 0.0, 0.0};

        public Builder text(String text) {
            this.text = text;
            return this;
        }

        public Builder position(double x, double y) {
            this.x = x;
            this.y = y;
            return this;
        }

        public Builder fontSize(double fontSize) {
            this.fontSize = fontSize;
            return this;
        }

        public Builder fontName(String fontName) {
            this.fontName = fontName;
            return this;
        }

        public Builder color(double r, double g, double b) {
            this.color = new double[]{r, g, b};
            return this;
        }

        public TextContent build() {
            return new TextContent(this);
        }
    }
}
""",

    "dom/ImageContent.java": """package com.pdfoxide.dom;

/**
 * Image content model for PDF pages.
 */
public final class ImageContent {
    private final double x;
    private final double y;
    private final double width;
    private final double height;
    private final String path;

    private ImageContent(Builder builder) {
        this.x = builder.x;
        this.y = builder.y;
        this.width = builder.width;
        this.height = builder.height;
        this.path = builder.path;
    }

    public static Builder builder() {
        return new Builder();
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

    public String getPath() {
        return path;
    }

    public static final class Builder {
        private double x;
        private double y;
        private double width;
        private double height;
        private String path;

        public Builder position(double x, double y) {
            this.x = x;
            this.y = y;
            return this;
        }

        public Builder width(double width) {
            this.width = width;
            return this;
        }

        public Builder height(double height) {
            this.height = height;
            return this;
        }

        public Builder path(String path) {
            this.path = path;
            return this;
        }

        public ImageContent build() {
            return new ImageContent(this);
        }
    }
}
""",

    # Conversion classes
    "conversion/ConversionOptionsBuilder.java": """package com.pdfoxide.conversion;

/**
 * Builder for ConversionOptions.
 */
public final class ConversionOptionsBuilder {
    private boolean detectHeadings = true;
    private boolean preserveLayout = true;
    private boolean includeImages = true;
    private int jpegQuality = 85;
    private String outputEncoding = "UTF-8";

    public ConversionOptionsBuilder detectHeadings(boolean detect) {
        this.detectHeadings = detect;
        return this;
    }

    public ConversionOptionsBuilder preserveLayout(boolean preserve) {
        this.preserveLayout = preserve;
        return this;
    }

    public ConversionOptionsBuilder includeImages(boolean include) {
        this.includeImages = include;
        return this;
    }

    public ConversionOptionsBuilder jpegQuality(int quality) {
        this.jpegQuality = quality;
        return this;
    }

    public ConversionOptionsBuilder outputEncoding(String encoding) {
        this.outputEncoding = encoding;
        return this;
    }

    public ConversionOptions build() {
        return new ConversionOptions(detectHeadings, preserveLayout, includeImages, jpegQuality, outputEncoding);
    }
}
""",

    # Additional DOM support classes
    "dom/TextStyle.java": """package com.pdfoxide.dom;

/**
 * Text style information.
 */
public final class TextStyle {
    private final String fontName;
    private final double fontSize;
    private final boolean bold;
    private final boolean italic;
    private final double[] color;

    public TextStyle(String fontName, double fontSize, boolean bold, boolean italic, double[] color) {
        this.fontName = fontName;
        this.fontSize = fontSize;
        this.bold = bold;
        this.italic = italic;
        this.color = color;
    }

    public String getFontName() {
        return fontName;
    }

    public double getFontSize() {
        return fontSize;
    }

    public boolean isBold() {
        return bold;
    }

    public boolean isItalic() {
        return italic;
    }

    public double[] getColor() {
        return color;
    }
}
""",

    "dom/PageMetrics.java": """package com.pdfoxide.dom;

/**
 * Page size and metrics information.
 */
public final class PageMetrics {
    private final double width;
    private final double height;
    private final int rotation;

    public PageMetrics(double width, double height, int rotation) {
        this.width = width;
        this.height = height;
        this.rotation = rotation;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    public int getRotation() {
        return rotation;
    }
}
""",

    # Annotation action classes
    "annotations/AnnotationAction.java": """package com.pdfoxide.annotations;

/**
 * Base class for annotation actions.
 */
public abstract class AnnotationAction {
    public abstract String getActionType();
}
""",

    "annotations/LaunchAction.java": """package com.pdfoxide.annotations;

/**
 * Launch action for annotations.
 */
public final class LaunchAction extends AnnotationAction {
    private final String filePath;
    private final String parameters;

    public LaunchAction(String filePath, String parameters) {
        this.filePath = filePath;
        this.parameters = parameters;
    }

    public String getFilePath() {
        return filePath;
    }

    public String getParameters() {
        return parameters;
    }

    @Override
    public String getActionType() {
        return "Launch";
    }
}
""",

    "annotations/GoToAction.java": """package com.pdfoxide.annotations;

/**
 * GoTo action for annotations.
 */
public final class GoToAction extends AnnotationAction {
    private final int pageIndex;
    private final double x;
    private final double y;
    private final double zoom;

    public GoToAction(int pageIndex, double x, double y, double zoom) {
        this.pageIndex = pageIndex;
        this.x = x;
        this.y = y;
        this.zoom = zoom;
    }

    public int getPageIndex() {
        return pageIndex;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getZoom() {
        return zoom;
    }

    @Override
    public String getActionType() {
        return "GoTo";
    }
}
""",

    # Exception helpers
    "exceptions/ExceptionUtils.java": """package com.pdfoxide.exceptions;

/**
 * Utility methods for exception handling.
 */
public final class ExceptionUtils {
    private ExceptionUtils() {
    }

    /**
     * Throws appropriate exception for error code.
     *
     * @param errorCode error code from native
     * @param message error message
     * @throws PdfException always
     */
    public static void throwException(int errorCode, String message) throws PdfException {
        switch (errorCode) {
            case 1:
                throw new ParseException(message);
            case 2:
                throw new EncryptionException(message);
            case 3:
                throw new IoException(message);
            case 4:
                throw new InvalidStateException(message);
            case 5:
                throw new UnsupportedFeatureException(message);
            default:
                throw new PdfException(message);
        }
    }
}
""",

    # Annotation types that were missing
    "annotations/UnderlineAnnotation.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Underline annotation.
 */
public final class UnderlineAnnotation extends Annotation {
    private final double[] color;

    public UnderlineAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "Underline";
    }

    public double[] getColor() {
        return color;
    }
}
""",

    "annotations/StrikeOutAnnotation.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Strike-out annotation.
 */
public final class StrikeOutAnnotation extends Annotation {
    private final double[] color;

    public StrikeOutAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "StrikeOut";
    }

    public double[] getColor() {
        return color;
    }
}
""",

    "annotations/SquigglyAnnotation.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Squiggly annotation.
 */
public final class SquigglyAnnotation extends Annotation {
    private final double[] color;

    public SquigglyAnnotation(Rect rect, double[] color) {
        super(rect);
        this.color = color;
    }

    @Override
    public String getType() {
        return "Squiggly";
    }

    public double[] getColor() {
        return color;
    }
}
""",

    "annotations/Caret.java": """package com.pdfoxide.annotations;

/**
 * Caret symbol types.
 */
public enum Caret {
    P, // Paragraph
    NONE;

    @Override
    public String toString() {
        return name().equals("NONE") ? "None" : name();
    }
}
""",

    "annotations/FileAttachmentIcon.java": """package com.pdfoxide.annotations;

/**
 * File attachment annotation icons.
 */
public enum FileAttachmentIcon {
    GRAPH, PAPERCLIP, PUSH_PIN, TAG
}
""",

    "annotations/PolygonAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for polygon annotations.
 */
public final class PolygonAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private PolygonAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static PolygonAnnotationBuilder create(Rect rect) {
        return new PolygonAnnotationBuilder(rect);
    }

    public PolygonAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public PolygonAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public PolygonAnnotation build() {
        return new PolygonAnnotation(rect, color, lineWidth);
    }
}
""",

    "annotations/InkAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for ink annotations.
 */
public final class InkAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private InkAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static InkAnnotationBuilder create(Rect rect) {
        return new InkAnnotationBuilder(rect);
    }

    public InkAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public InkAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public InkAnnotation build() {
        return new InkAnnotation(rect, color, lineWidth);
    }
}
""",

    "annotations/RedactAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for redact annotations.
 */
public final class RedactAnnotationBuilder {
    private final Rect rect;
    private double[] color = new double[]{0.0, 0.0, 0.0};
    private String replacementText;

    private RedactAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static RedactAnnotationBuilder create(Rect rect) {
        return new RedactAnnotationBuilder(rect);
    }

    public RedactAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public RedactAnnotationBuilder replacementText(String text) {
        this.replacementText = text;
        return this;
    }

    public RedactAnnotation build() {
        return new RedactAnnotation(rect, color, replacementText);
    }
}
""",

    # Geometry classes
    "geometry/Transform.java": """package com.pdfoxide.geometry;

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
""",

    "geometry/Matrix.java": """package com.pdfoxide.geometry;

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
""",
}

# Generate all files
for filepath, content in files.items():
    full_path = os.path.join(base_dir, filepath)
    os.makedirs(os.path.dirname(full_path), exist_ok=True)
    with open(full_path, 'w') as f:
        f.write(content)
    print(f"Generated: {filepath}")

print(f"\\nGenerated {len(files)} more Java files")

PYTHON_EOF

echo "Phase 3 of class generation complete!"
