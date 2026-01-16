#!/bin/bash
# Generate missing annotation builder classes and form field classes

BASE_DIR="/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

python3 << 'PYTHON_EOF'
import os

base_dir = "/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

# Files for annotation builders and remaining annotations
files = {
    # Annotation Builders
    "annotations/TextAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for text annotations.
 */
public final class TextAnnotationBuilder {
    private final Rect rect;
    private final String content;
    private String author;
    private double[] color;

    private TextAnnotationBuilder(Rect rect, String content) {
        this.rect = rect;
        this.content = content;
    }

    public static TextAnnotationBuilder create(Rect rect, String content) {
        return new TextAnnotationBuilder(rect, content);
    }

    public TextAnnotationBuilder author(String author) {
        this.author = author;
        return this;
    }

    public TextAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public TextAnnotation build() {
        return new TextAnnotation(rect, content, author, color);
    }
}
""",

    "annotations/HighlightAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for highlight annotations.
 */
public final class HighlightAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double opacity = 1.0;

    private HighlightAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static HighlightAnnotationBuilder create(Rect rect) {
        return new HighlightAnnotationBuilder(rect);
    }

    public HighlightAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public HighlightAnnotationBuilder opacity(double opacity) {
        this.opacity = opacity;
        return this;
    }

    public HighlightAnnotation build() {
        return new HighlightAnnotation(rect, color, opacity);
    }
}
""",

    "annotations/LinkAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for link annotations.
 */
public final class LinkAnnotationBuilder {
    private final Rect rect;
    private final LinkAction action;

    private LinkAnnotationBuilder(Rect rect, LinkAction action) {
        this.rect = rect;
        this.action = action;
    }

    public static LinkAnnotationBuilder create(Rect rect, LinkAction action) {
        return new LinkAnnotationBuilder(rect, action);
    }

    public LinkAnnotation build() {
        return new LinkAnnotation(rect, action);
    }
}
""",

    "annotations/LinkAction.java": """package com.pdfoxide.annotations;

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
""",

    "annotations/StampAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for stamp annotations.
 */
public final class StampAnnotationBuilder {
    private final Rect rect;
    private final StampType stampType;
    private double[] color;

    private StampAnnotationBuilder(Rect rect, StampType stampType) {
        this.rect = rect;
        this.stampType = stampType;
    }

    public static StampAnnotationBuilder create(Rect rect, StampType stampType) {
        return new StampAnnotationBuilder(rect, stampType);
    }

    public StampAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public StampAnnotation build() {
        return new StampAnnotation(rect, stampType, color);
    }
}
""",

    "annotations/WatermarkAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for watermark annotations.
 */
public final class WatermarkAnnotationBuilder {
    private final Rect rect;
    private final String text;
    private double opacity = 0.5;
    private double[] color;

    private WatermarkAnnotationBuilder(Rect rect, String text) {
        this.rect = rect;
        this.text = text;
    }

    public static WatermarkAnnotationBuilder create(Rect rect, String text) {
        return new WatermarkAnnotationBuilder(rect, text);
    }

    public WatermarkAnnotationBuilder opacity(double opacity) {
        this.opacity = opacity;
        return this;
    }

    public WatermarkAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public WatermarkAnnotation build() {
        return new WatermarkAnnotation(rect, text, opacity, color);
    }
}
""",

    "annotations/LineAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for line annotations.
 */
public final class LineAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private LineAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static LineAnnotationBuilder create(Rect rect) {
        return new LineAnnotationBuilder(rect);
    }

    public LineAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public LineAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public LineAnnotation build() {
        return new LineAnnotation(rect, color, lineWidth);
    }
}
""",

    "annotations/SquareAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for square annotations.
 */
public final class SquareAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private SquareAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static SquareAnnotationBuilder create(Rect rect) {
        return new SquareAnnotationBuilder(rect);
    }

    public SquareAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public SquareAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public SquareAnnotation build() {
        return new SquareAnnotation(rect, color, lineWidth);
    }
}
""",

    "annotations/CircleAnnotationBuilder.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for circle annotations.
 */
public final class CircleAnnotationBuilder {
    private final Rect rect;
    private double[] color;
    private double lineWidth = 1.0;

    private CircleAnnotationBuilder(Rect rect) {
        this.rect = rect;
    }

    public static CircleAnnotationBuilder create(Rect rect) {
        return new CircleAnnotationBuilder(rect);
    }

    public CircleAnnotationBuilder color(double r, double g, double b) {
        this.color = new double[]{r, g, b};
        return this;
    }

    public CircleAnnotationBuilder lineWidth(double width) {
        this.lineWidth = width;
        return this;
    }

    public CircleAnnotation build() {
        return new CircleAnnotation(rect, color, lineWidth);
    }
}
""",

    # Additional annotation classes
    "annotations/SquareAnnotation.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Square annotation.
 */
public final class SquareAnnotation extends Annotation {
    private final double[] color;
    private final double lineWidth;

    public SquareAnnotation(Rect rect, double[] color, double lineWidth) {
        super(rect);
        this.color = color;
        this.lineWidth = lineWidth;
    }

    @Override
    public String getType() {
        return "Square";
    }

    public double[] getColor() {
        return color;
    }

    public double getLineWidth() {
        return lineWidth;
    }
}
""",

    "annotations/CircleAnnotation.java": """package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Circle annotation.
 */
public final class CircleAnnotation extends Annotation {
    private final double[] color;
    private final double lineWidth;

    public CircleAnnotation(Rect rect, double[] color, double lineWidth) {
        super(rect);
        this.color = color;
        this.lineWidth = lineWidth;
    }

    @Override
    public String getType() {
        return "Circle";
    }

    public double[] getColor() {
        return color;
    }

    public double getLineWidth() {
        return lineWidth;
    }
}
""",

    # Form field builders
    "forms/TextFieldBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for text fields.
 */
public final class TextFieldBuilder {
    private final String name;
    private final Rect rect;
    private String defaultValue;
    private int maxLength = -1;
    private boolean required = false;
    private boolean readOnly = false;

    private TextFieldBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static TextFieldBuilder create(String name, Rect rect) {
        return new TextFieldBuilder(name, rect);
    }

    public TextFieldBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public TextFieldBuilder maxLength(int length) {
        this.maxLength = length;
        return this;
    }

    public TextFieldBuilder required(boolean required) {
        this.required = required;
        return this;
    }

    public TextFieldBuilder readOnly(boolean readOnly) {
        this.readOnly = readOnly;
        return this;
    }

    public TextField build() {
        return new TextField(name, rect, defaultValue, maxLength, required, readOnly);
    }
}
""",

    "forms/CheckboxBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for checkbox fields.
 */
public final class CheckboxBuilder {
    private final String name;
    private final Rect rect;
    private boolean defaultChecked = false;
    private String exportValue = "Yes";

    private CheckboxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static CheckboxBuilder create(String name, Rect rect) {
        return new CheckboxBuilder(name, rect);
    }

    public CheckboxBuilder defaultChecked(boolean checked) {
        this.defaultChecked = checked;
        return this;
    }

    public CheckboxBuilder exportValue(String value) {
        this.exportValue = value;
        return this;
    }

    public CheckboxField build() {
        return new CheckboxField(name, rect, defaultChecked, exportValue);
    }
}
""",

    "forms/ComboBoxBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Arrays;
import java.util.List;

/**
 * Builder for combo box fields.
 */
public final class ComboBoxBuilder {
    private final String name;
    private final Rect rect;
    private List<String> options;
    private boolean editable = false;
    private String defaultValue;

    private ComboBoxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static ComboBoxBuilder create(String name, Rect rect) {
        return new ComboBoxBuilder(name, rect);
    }

    public ComboBoxBuilder options(String... options) {
        this.options = Arrays.asList(options);
        return this;
    }

    public ComboBoxBuilder options(List<String> options) {
        this.options = options;
        return this;
    }

    public ComboBoxBuilder editable(boolean editable) {
        this.editable = editable;
        return this;
    }

    public ComboBoxBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public ComboBoxField build() {
        return new ComboBoxField(name, rect, options, editable, defaultValue);
    }
}
""",

    "forms/ListBoxBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.Arrays;
import java.util.List;

/**
 * Builder for list box fields.
 */
public final class ListBoxBuilder {
    private final String name;
    private final Rect rect;
    private List<String> options;
    private boolean multiSelect = false;
    private List<String> selectedValues;

    private ListBoxBuilder(String name, Rect rect) {
        this.name = name;
        this.rect = rect;
    }

    public static ListBoxBuilder create(String name, Rect rect) {
        return new ListBoxBuilder(name, rect);
    }

    public ListBoxBuilder options(String... options) {
        this.options = Arrays.asList(options);
        return this;
    }

    public ListBoxBuilder options(List<String> options) {
        this.options = options;
        return this;
    }

    public ListBoxBuilder multiSelect(boolean multiSelect) {
        this.multiSelect = multiSelect;
        return this;
    }

    public ListBoxBuilder selectedValues(List<String> values) {
        this.selectedValues = values;
        return this;
    }

    public ListBoxField build() {
        return new ListBoxField(name, rect, options, multiSelect, selectedValues);
    }
}
""",

    "forms/RadioButtonGroupBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;
import java.util.ArrayList;
import java.util.List;

/**
 * Builder for radio button groups.
 */
public final class RadioButtonGroupBuilder {
    private final String name;
    private final List<RadioButtonBuilder.Option> options = new ArrayList<>();
    private String defaultValue;

    private RadioButtonGroupBuilder(String name) {
        this.name = name;
    }

    public static RadioButtonGroupBuilder create(String name) {
        return new RadioButtonGroupBuilder(name);
    }

    public RadioButtonGroupBuilder addButton(Rect rect, String value) {
        options.add(new RadioButtonBuilder.Option(rect, value));
        return this;
    }

    public RadioButtonGroupBuilder defaultValue(String value) {
        this.defaultValue = value;
        return this;
    }

    public RadioButtonField build() {
        return new RadioButtonField(name, options, defaultValue);
    }
}
""",

    "forms/RadioButtonBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Helper for radio button definitions.
 */
public final class RadioButtonBuilder {
    public static final class Option {
        private final Rect rect;
        private final String value;

        public Option(Rect rect, String value) {
            this.rect = rect;
            this.value = value;
        }

        public Rect getRect() {
            return rect;
        }

        public String getValue() {
            return value;
        }
    }
}
""",

    "forms/PushButtonBuilder.java": """package com.pdfoxide.forms;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for push button fields.
 */
public final class PushButtonBuilder {
    private final String name;
    private final Rect rect;
    private final String label;
    private ButtonAction action;

    private PushButtonBuilder(String name, Rect rect, String label) {
        this.name = name;
        this.rect = rect;
        this.label = label;
    }

    public static PushButtonBuilder create(String name, Rect rect, String label) {
        return new PushButtonBuilder(name, rect, label);
    }

    public PushButtonBuilder action(ButtonAction action) {
        this.action = action;
        return this;
    }

    public PushButtonField build() {
        return new PushButtonField(name, rect, label, action);
    }
}
""",

    "forms/ButtonAction.java": """package com.pdfoxide.forms;

/**
 * Actions for push buttons.
 */
public final class ButtonAction {
    private final String type;
    private final String target;

    private ButtonAction(String type, String target) {
        this.type = type;
        this.target = target;
    }

    /**
     * Creates a submit action.
     *
     * @param url submit URL
     * @return button action
     */
    public static ButtonAction submit(String url) {
        return new ButtonAction("submit", url);
    }

    /**
     * Creates a reset action.
     *
     * @return button action
     */
    public static ButtonAction reset() {
        return new ButtonAction("reset", "");
    }

    /**
     * Creates a custom action.
     *
     * @param type action type
     * @param target action target
     * @return button action
     */
    public static ButtonAction custom(String type, String target) {
        return new ButtonAction(type, target);
    }

    public String getType() {
        return type;
    }

    public String getTarget() {
        return target;
    }
}
""",

    "forms/FormExtractor.java": """package com.pdfoxide.forms;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;
import java.util.List;

/**
 * Extracts form fields from PDF documents.
 */
public final class FormExtractor {
    private final PdfDocument document;

    public FormExtractor(PdfDocument document) {
        this.document = document;
    }

    /**
     * Extracts all form fields.
     *
     * @return list of form fields
     * @throws PdfException if extraction fails
     */
    public List<FormField> extractFields() throws PdfException {
        return nativeExtractFields(document);
    }

    /**
     * Exports form data to FDF format.
     *
     * @param path output file path
     * @throws PdfException if export fails
     */
    public void exportFdf(String path) throws PdfException {
        nativeExportFdf(document, path);
    }

    /**
     * Exports form data to XFDF format.
     *
     * @param path output file path
     * @throws PdfException if export fails
     */
    public void exportXfdf(String path) throws PdfException {
        nativeExportXfdf(document, path);
    }

    public void close() throws PdfException {
        // Cleanup if needed
    }

    private static native List<FormField> nativeExtractFields(PdfDocument document) throws PdfException;
    private static native void nativeExportFdf(PdfDocument document, String path) throws PdfException;
    private static native void nativeExportXfdf(PdfDocument document, String path) throws PdfException;
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

echo "Phase 2 of class generation complete!"
