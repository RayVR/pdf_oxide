#!/bin/bash

BASE_DIR="/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

python3 << 'PYTHON_EOF'
import os

base_dir = "/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

files = {
    # Missing annotation builders
    "annotations/CaretAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for caret annotations.
 */
public final class CaretAnnotationBuilder {
    private final Rect rect;
    private Caret caretType = Caret.P;
    private double[] color;

    private CaretAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static CaretAnnotationBuilder create(Rect rect) {
        return new CaretAnnotationBuilder(rect);
    }

    public CaretAnnotationBuilder caretType(Caret type) {
        this.caretType = type;
        return this;
    }

    public CaretAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public CaretAnnotation build() {
        return new CaretAnnotation(rect, caretType, color);
    }
}
""",

    "annotations/FreeTextAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for free text annotations.
 */
public final class FreeTextAnnotationBuilder {
    private final Rect rect;
    private final String content;
    private String fontName = "Helvetica";
    private double fontSize = 12.0;
    private double[] color;

    private FreeTextAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static FreeTextAnnotationBuilder create(Rect rect, String content) {
        return new FreeTextAnnotationBuilder(rect, content);
    }

    public FreeTextAnnotationBuilder fontName(String fontName) {
        this.fontName = fontName;
        return this;
    }

    public FreeTextAnnotationBuilder fontSize(double fontSize) {
        this.fontSize = fontSize;
        return this;
    }

    public FreeTextAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public FreeTextAnnotation build() {
        return new FreeTextAnnotation(rect, content, fontName, fontSize, color);
    }
}
""",

    "annotations/FileAttachmentAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for file attachment annotations.
 */
public final class FileAttachmentAnnotationBuilder {
    private final Rect rect;
    private final String filePath;
    private String fileName;
    private FileAttachmentIcon icon = FileAttachmentIcon.PUSH_PIN;

    private FileAttachmentAnnotationBuilder(Rect rect, String filePath) {
        this.rect = rect;
        this.filePath = filePath;
    }

    public static FileAttachmentAnnotationBuilder create(Rect rect, String filePath) {
        return new FileAttachmentAnnotationBuilder(rect, filePath);
    }

    public FileAttachmentAnnotationBuilder fileName(String name) {
        this.fileName = name;
        return this;
    }

    public FileAttachmentAnnotationBuilder icon(FileAttachmentIcon icon) {
        this.icon = icon;
        return this;
    }

    public FileAttachmentAnnotation build() {
        return new FileAttachmentAnnotation(rect, filePath, fileName, icon);
    }
}
""",

    "annotations/SoundAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for sound annotations.
 */
public final class SoundAnnotationBuilder {
    private final Rect rect;
    private final String audioPath;

    private SoundAnnotationBuilder(Rect rect, String audioPath) {
        this.rect = rect;
        this.audioPath = audioPath;
    }

    public static SoundAnnotationBuilder create(Rect rect, String audioPath) {
        return new SoundAnnotationBuilder(rect, audioPath);
    }

    public SoundAnnotation build() {
        return new SoundAnnotation(rect, audioPath);
    }
}
""",

    "annotations/MovieAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for movie annotations.
 */
public final class MovieAnnotationBuilder {
    private final Rect rect;
    private final String moviePath;

    private MovieAnnotationBuilder(Rect rect, String moviePath) {
        this.rect = rect;
        this.moviePath = moviePath;
    }

    public static MovieAnnotationBuilder create(Rect rect, String moviePath) {
        return new MovieAnnotationBuilder(rect, moviePath);
    }

    public MovieAnnotation build() {
        return new MovieAnnotation(rect, moviePath);
    }
}
""",

    "annotations/ScreenAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for screen annotations.
 */
public final class ScreenAnnotationBuilder {
    private final Rect rect;
    private final String mediaPath;

    private ScreenAnnotationBuilder(Rect rect, String mediaPath) {
        this.rect = rect;
        this.mediaPath = mediaPath;
    }

    public static ScreenAnnotationBuilder create(Rect rect, String mediaPath) {
        return new ScreenAnnotationBuilder(rect, mediaPath);
    }

    public ScreenAnnotation build() {
        return new ScreenAnnotation(rect, mediaPath);
    }
}
""",

    "annotations/RichMediaAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for rich media annotations.
 */
public final class RichMediaAnnotationBuilder {
    private final Rect rect;
    private final String mediaPath;

    private RichMediaAnnotationBuilder(Rect rect, String mediaPath) {
        this.rect = rect;
        this.mediaPath = mediaPath;
    }

    public static RichMediaAnnotationBuilder create(Rect rect, String mediaPath) {
        return new RichMediaAnnotationBuilder(rect, mediaPath);
    }

    public RichMediaAnnotation build() {
        return new RichMediaAnnotation(rect, mediaPath);
    }
}
""",

    "annotations/ThreeDAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for 3D annotations.
 */
public final class ThreeDAnnotationBuilder {
    private final Rect rect;
    private final String u3dPath;

    private ThreeDAnnotationBuilder(Rect rect, String u3dPath) {
        this.rect = rect;
        this.u3dPath = u3dPath;
    }

    public static ThreeDAnnotationBuilder create(Rect rect, String u3dPath) {
        return new ThreeDAnnotationBuilder(rect, u3dPath);
    }

    public ThreeDAnnotation build() {
        return new ThreeDAnnotation(rect, u3dPath);
    }
}
""",

    "annotations/PopupAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for popup annotations.
 */
public final class PopupAnnotationBuilder {
    private final Rect rect;
    private final String content;

    private PopupAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static PopupAnnotationBuilder create(Rect rect, String content) {
        return new PopupAnnotationBuilder(rect, content);
    }

    public PopupAnnotation build() {
        return new PopupAnnotation(rect, content);
    }
}
""",

    # Form field builders for signature
    "forms/SignatureFieldBuilder.java": """package com.pdfoxide.forms;

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
""",

    # Additional utility and support classes
    "util/PdfVersion.java": """package com.pdfoxide.util;

/**
 * PDF version information.
 */
public final class PdfVersion {
    private final int major;
    private final int minor;

    public PdfVersion(int major, int minor) {
        this.major = major;
        this.minor = minor;
    }

    public int getMajor() {
        return major;
    }

    public int getMinor() {
        return minor;
    }

    @Override
    public String toString() {
        return major + "." + minor;
    }
}
""",

    # Conversion helper
    "conversion/MarkdownOptions.java": """package com.pdfoxide.conversion;

/**
 * Options specific to Markdown conversion.
 */
public final class MarkdownOptions {
    private final boolean detectHeadings;
    private final boolean preserveLayout;
    private final boolean createTableOfContents;

    public MarkdownOptions(boolean detectHeadings, boolean preserveLayout, boolean createTableOfContents) {
        this.detectHeadings = detectHeadings;
        this.preserveLayout = preserveLayout;
        this.createTableOfContents = createTableOfContents;
    }

    public boolean isDetectHeadings() {
        return detectHeadings;
    }

    public boolean isPreserveLayout() {
        return preserveLayout;
    }

    public boolean isCreateTableOfContents() {
        return createTableOfContents;
    }
}
""",

    "conversion/HtmlOptions.java": """package com.pdfoxide.conversion;

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
""",

    # Security configuration
    "security/SignatureConfigBuilder.java": """package com.pdfoxide.security;

/**
 * Builder for signature configuration.
 */
public final class SignatureConfigBuilder {
    private byte[] certificate;
    private byte[] privateKey;
    private String reason;
    private String location;
    private String contactInfo;

    public SignatureConfigBuilder certificate(byte[] cert) {
        this.certificate = cert;
        return this;
    }

    public SignatureConfigBuilder privateKey(byte[] key) {
        this.privateKey = key;
        return this;
    }

    public SignatureConfigBuilder reason(String reason) {
        this.reason = reason;
        return this;
    }

    public SignatureConfigBuilder location(String location) {
        this.location = location;
        return this;
    }

    public SignatureConfigBuilder contactInfo(String info) {
        this.contactInfo = info;
        return this;
    }

    public SignatureConfig build() {
        return new SignatureConfig(certificate, privateKey, reason, location, contactInfo);
    }
}
""",

    # Content types
    "dom/ContentType.java": """package com.pdfoxide.dom;

/**
 * Types of content in PDF pages.
 */
public enum ContentType {
    TEXT,
    IMAGE,
    SHAPE,
    ANNOTATION,
    FORM_FIELD,
    TABLE,
    PATH,
    GRAPHICS_STATE,
    UNKNOWN
}
""",

    "dom/TableCell.java": """package com.pdfoxide.dom;

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
""",

    # Geometry helpers
    "geometry/Dimensions.java": """package com.pdfoxide.geometry;

/**
 * Width and height dimensions.
 */
public final class Dimensions {
    private final double width;
    private final double height;

    public Dimensions(double width, double height) {
        this.width = width;
        this.height = height;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    @Override
    public String toString() {
        return "Dimensions{" +
                "width=" + width +
                ", height=" + height +
                '}';
    }
}
""",

    "geometry/Margin.java": """package com.pdfoxide.geometry;

/**
 * Margin values.
 */
public final class Margin {
    private final double top;
    private final double right;
    private final double bottom;
    private final double left;

    public Margin(double top, double right, double bottom, double left) {
        this.top = top;
        this.right = right;
        this.bottom = bottom;
        this.left = left;
    }

    public double getTop() { return top; }
    public double getRight() { return right; }
    public double getBottom() { return bottom; }
    public double getLeft() { return left; }

    public static Margin uniform(double all) {
        return new Margin(all, all, all, all);
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

print(f"\\nGenerated {len(files)} final Java files")

PYTHON_EOF

echo "Final phase of class generation complete!"
