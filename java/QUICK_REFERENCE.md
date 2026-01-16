# Java API Quick Reference

## Imports Cheat Sheet

```java
// Core API
import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.core.PdfBuilder;

// Document Editing
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.document.DocumentInfo;

// DOM Navigation
import com.pdfoxide.dom.*;

// Annotations
import com.pdfoxide.annotations.*;

// Form Fields
import com.pdfoxide.forms.*;

// Search
import com.pdfoxide.search.*;

// Compliance
import com.pdfoxide.compliance.*;

// Conversion
import com.pdfoxide.conversion.*;

// Geometry
import com.pdfoxide.geometry.*;

// Utilities
import com.pdfoxide.util.*;
```

## Common Patterns

### 1. Reading PDF Files

```java
// Simple read
try (PdfDocument doc = PdfDocument.open("input.pdf")) {
    int pages = doc.getPageCount();
    String text = doc.extractText(0);
}

// With conversion
try (PdfDocument doc = PdfDocument.open("input.pdf")) {
    String markdown = doc.toMarkdown(0);
    String html = doc.toHtml(0);
}
```

### 2. Creating PDFs

```java
// From Markdown
Pdf doc = Pdf.fromMarkdown("# Title\n\nContent");
doc.save("output.pdf");
doc.close();

// From HTML
Pdf doc = Pdf.fromHtml("<h1>Title</h1><p>Content</p>");
doc.save("output.pdf");
doc.close();

// From Image
Pdf doc = Pdf.fromImage("image.png");
doc.save("output.pdf");
doc.close();

// With Configuration
Pdf doc = PdfBuilder.create()
    .title("My PDF")
    .author("John Doe")
    .subject("Example")
    .keywords("pdf", "example")
    .fromMarkdown("Content");
doc.save("output.pdf");
doc.close();
```

### 3. DOM Navigation

```java
Pdf doc = Pdf.open("input.pdf");

// Get page
PdfPage page = doc.getPage(0);

// Find elements
List<PdfText> texts = page.findTextContaining("search term");
List<PdfImage> images = page.findImages();
List<PdfTable> tables = page.findTables();

doc.close();
```

### 4. Adding Annotations

```java
DocumentEditor editor = DocumentEditor.open("input.pdf");

// Comment
TextAnnotation comment = TextAnnotationBuilder.create(
    new Rect(100, 700, 150, 20),
    "This is a note"
)
    .author("John Doe")
    .color(1.0, 1.0, 0.0)  // Yellow
    .build();
editor.addAnnotation(0, comment);

// Highlight
HighlightAnnotation highlight = HighlightAnnotationBuilder.create(
    new Rect(200, 650, 300, 30)
)
    .color(1.0, 1.0, 0.0)
    .build();
editor.addAnnotation(0, highlight);

// Link
LinkAnnotation link = LinkAnnotationBuilder.create(
    new Rect(400, 600, 500, 30),
    LinkAction.externalLink("https://example.com")
)
    .build();
editor.addAnnotation(0, link);

// Stamp
StampAnnotation stamp = StampAnnotationBuilder.create(
    new Rect(100, 500, 150, 30),
    StampType.APPROVED
)
    .color(0.0, 1.0, 0.0)  // Green
    .build();
editor.addAnnotation(0, stamp);

editor.save("output.pdf");
editor.close();
```

### 5. Creating Form Fields

```java
DocumentEditor editor = DocumentEditor.open("input.pdf");

// Text field
TextField name = TextFieldBuilder.create("name", new Rect(100, 700, 300, 20))
    .required(true)
    .maxLength(100)
    .defaultValue("John Doe")
    .build();
editor.addFormField(0, name);

// Checkbox
CheckboxField agree = CheckboxBuilder.create("agree", new Rect(100, 670, 15, 15))
    .defaultChecked(false)
    .exportValue("yes")
    .build();
editor.addFormField(0, agree);

// Dropdown
ComboBoxField country = ComboBoxBuilder.create("country", new Rect(100, 640, 150, 20))
    .options("USA", "Canada", "Mexico")
    .editable(false)
    .build();
editor.addFormField(0, country);

// List box
ListBoxField skills = ListBoxBuilder.create("skills", new Rect(100, 600, 200, 60))
    .options("Java", "Python", "Rust", "Go")
    .multiSelect(true)
    .build();
editor.addFormField(0, skills);

// Radio buttons
RadioButtonField priority = RadioButtonGroupBuilder.create("priority")
    .addButton(new Rect(100, 550, 15, 15), "high")
    .addButton(new Rect(130, 550, 15, 15), "medium")
    .addButton(new Rect(160, 550, 15, 15), "low")
    .defaultValue("medium")
    .build();
editor.addFormField(0, priority);

// Submit button
PushButtonField submit = PushButtonBuilder.create("submit", new Rect(100, 500, 100, 30), "Submit")
    .action(ButtonAction.submit("http://example.com/submit"))
    .build();
editor.addFormField(0, submit);

editor.save("output.pdf");
editor.close();
```

### 6. Text Search

```java
try (PdfDocument doc = PdfDocument.open("input.pdf")) {
    TextSearcher searcher = new TextSearcher(doc);

    // Basic search
    SearchOptions options = SearchOptions.builder()
        .caseSensitive(false)
        .wholeWord(false)
        .build();

    List<SearchResult> results = searcher.search("example", options);

    for (SearchResult result : results) {
        System.out.printf("Found on page %d at (%f, %f): \"%s\"%n",
            result.getPage() + 1,
            result.getX(),
            result.getY(),
            result.getText());
    }

    searcher.close();
}
```

### 7. PDF/A Compliance

```java
try (PdfDocument doc = PdfDocument.open("input.pdf")) {
    // Validate against PDF/A-2B
    PdfAValidator validator = new PdfAValidator(PdfALevel.LEVEL_2B);
    ValidationResult result = validator.validate(doc);

    if (result.isValid()) {
        System.out.println("✓ Document is compliant");
    } else {
        System.out.println("✗ Document has issues:");

        for (ComplianceError error : result.getErrors()) {
            System.out.printf("ERROR: %s - %s%n",
                error.getCode(),
                error.getMessage());
        }

        for (ComplianceWarning warning : result.getWarnings()) {
            System.out.printf("WARNING: %s%n", warning.getMessage());
        }
    }

    ValidationStats stats = result.getStats();
    System.out.printf("Pages: %d, Elements: %d, Time: %dms%n",
        stats.getPagesChecked(),
        stats.getElementsAnalyzed(),
        stats.getValidationTime());
}
```

### 8. Digital Signatures

```java
try (PdfDocument doc = PdfDocument.open("input.pdf")) {
    // Check signatures
    int count = DigitalSignature.getSignatureCount(doc);
    System.out.printf("Document has %d signature(s)%n", count);

    // Get signature info
    for (int i = 0; i < count; i++) {
        DigitalSignature sig = DigitalSignature.getSignature(doc, i);
        System.out.printf("Signature %d:%n", i + 1);
        System.out.printf("  Name: %s%n", sig.getName());
        System.out.printf("  Reason: %s%n", sig.getReason());
        System.out.printf("  Location: %s%n", sig.getLocation());
        System.out.printf("  Date: %s%n", sig.getDate());
    }
}
```

### 9. Exception Handling

```java
try {
    PdfDocument doc = PdfDocument.open("invalid.pdf");
} catch (ParseException e) {
    System.err.println("Not a valid PDF: " + e.getMessage());
} catch (EncryptionException e) {
    System.err.println("Document is encrypted: " + e.getMessage());
} catch (IoException e) {
    System.err.println("File I/O error: " + e.getMessage());
} catch (PdfException e) {
    System.err.println("PDF error: " + e.getMessage());
}
```

### 10. Resource Management (Best Practices)

```java
// Pattern 1: Try-with-resources (RECOMMENDED)
try (PdfDocument doc = PdfDocument.open("file.pdf")) {
    // Use document
    System.out.println(doc.getPageCount());
} // Automatically closed

// Pattern 2: Manual management
PdfDocument doc = PdfDocument.open("file.pdf");
try {
    System.out.println(doc.getPageCount());
} finally {
    doc.close();
}

// Pattern 3: Builder with try-with-resources
try (PdfDocument doc = PdfBuilder.create()
        .title("Document")
        .fromMarkdown("# Title") as AutoCloseable) {
    // Use document
} // Automatically closed
```

## Common Annotation Types

```java
// Text/Comment
TextAnnotationBuilder.create(rect, "Comment text")
    .author("Author")
    .color(1.0, 1.0, 0.0)

// Highlight
HighlightAnnotationBuilder.create(rect)
    .color(1.0, 1.0, 0.0)
    .opacity(0.5)

// Link
LinkAnnotationBuilder.create(rect, LinkAction.externalLink("https://..."))
LinkAnnotationBuilder.create(rect, LinkAction.internalLink(2))

// Stamp
StampAnnotationBuilder.create(rect, StampType.APPROVED)
    .color(0.0, 1.0, 0.0)

// Watermark
WatermarkAnnotationBuilder.create(rect, "CONFIDENTIAL")
    .opacity(0.3)

// Redaction
RedactAnnotationBuilder.create(rect)
    .replacementText("[REDACTED]")

// Underline/Strikethrough/Squiggly
UnderlineAnnotation, StrikeOutAnnotation, SquigglyAnnotation

// Shapes
LineAnnotationBuilder, SquareAnnotationBuilder, CircleAnnotationBuilder, PolygonAnnotationBuilder

// Freehand
InkAnnotationBuilder

// Media
SoundAnnotationBuilder, MovieAnnotationBuilder, ScreenAnnotationBuilder
```

## Common Form Field Types

```java
// Text input
TextFieldBuilder.create("name", rect)
    .required(true)
    .maxLength(100)
    .defaultValue("value")

// Checkbox
CheckboxBuilder.create("agree", rect)
    .defaultChecked(false)
    .exportValue("yes")

// Dropdown
ComboBoxBuilder.create("choice", rect)
    .options("Option 1", "Option 2", "Option 3")
    .editable(true)

// List
ListBoxBuilder.create("items", rect)
    .options("Item 1", "Item 2", "Item 3")
    .multiSelect(true)

// Radio buttons
RadioButtonGroupBuilder.create("group")
    .addButton(rect1, "value1")
    .addButton(rect2, "value2")

// Button
PushButtonBuilder.create("submit", rect, "Click Me")
    .action(ButtonAction.submit("http://..."))

// Signature
SignatureFieldBuilder.create("sig", rect)
    .reason("Approval")
    .location("Office")
```

## Page Sizes

```java
PageSize.A0, PageSize.A1, PageSize.A2, PageSize.A3, PageSize.A4, PageSize.A5, PageSize.A6
PageSize.LETTER, PageSize.LEGAL, PageSize.TABLOID, PageSize.LEDGER
```

## PDF/A Levels

```java
PdfALevel.LEVEL_1A, PdfALevel.LEVEL_1B,
PdfALevel.LEVEL_2A, PdfALevel.LEVEL_2B, PdfALevel.LEVEL_2U,
PdfALevel.LEVEL_3A, PdfALevel.LEVEL_3B, PdfALevel.LEVEL_3U
```

## Conversion Options

```java
ConversionOptions options = ConversionOptions.builder()
    .detectHeadings(true)
    .preserveLayout(true)
    .includeImages(true)
    .jpegQuality(85)
    .outputEncoding("UTF-8")
    .build();

// Use with conversion
String markdown = doc.toMarkdown(0, options);
String html = doc.toHtml(0, options);
```

## Search Options

```java
SearchOptions options = SearchOptions.builder()
    .caseSensitive(false)
    .wholeWord(true)
    .useRegex(false)
    .maxResults(100)
    .pageIndex(0)  // Optional: limit to specific page
    .build();
```

## Geometry

```java
// Points
Point p = new Point(100.0, 200.0);

// Rectangles
Rect rect = new Rect(100.0, 200.0, 300.0, 400.0);  // x, y, width, height

// Colors
Color red = new Color(1.0, 0.0, 0.0);
Color green = new Color(0.0, 1.0, 0.0);
Color blue = new Color(0.0, 0.0, 1.0);

// Margins
Margin margin = Margin.uniform(72.0);
Margin customMargin = new Margin(72, 72, 72, 72);

// Dimensions
Dimensions dims = new Dimensions(612.0, 792.0);
```

## Complete Workflow Example

```java
public class PdfWorkflow {
    public static void main(String[] args) throws Exception {
        // 1. Create from Markdown
        Pdf created = PdfBuilder.create()
            .title("Report")
            .author("Developer")
            .fromMarkdown("# Report\n\nContent here");
        created.save("report.pdf");
        created.close();

        // 2. Add annotations and forms
        DocumentEditor editor = DocumentEditor.open("report.pdf");

        TextAnnotation note = TextAnnotationBuilder.create(
            new Rect(100, 700, 150, 20),
            "See section 2"
        ).build();
        editor.addAnnotation(0, note);

        TextField signature = TextFieldBuilder.create("sig", new Rect(100, 600, 300, 20))
            .required(true)
            .build();
        editor.addFormField(0, signature);

        editor.save("report_annotated.pdf");
        editor.close();

        // 3. Search content
        try (TextSearcher searcher = new TextSearcher(
                PdfDocument.open("report_annotated.pdf"))) {
            List<SearchResult> results = searcher.search(
                "section",
                SearchOptions.builder().caseSensitive(false).build()
            );
            System.out.println("Found " + results.size() + " matches");
        }

        // 4. Validate PDF/A
        try (PdfDocument doc = PdfDocument.open("report_annotated.pdf")) {
            ValidationResult result = new PdfAValidator(PdfALevel.LEVEL_2B).validate(doc);
            System.out.println("Valid: " + result.isValid());
        }
    }
}
```

---

**For more details, see PHASE_8_COMPLETION.md and API_EXAMPLES.md**
