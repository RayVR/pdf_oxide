# PDF Oxide Java API Examples

Complete code examples for all PDF Oxide Java binding features.

## Table of Contents

- [Phase 3: Universal API (Pdf)](#phase-3-universal-api-pdf)
- [Phase 4: DOM Navigation](#phase-4-dom-navigation)
- [Phase 5: Annotations](#phase-5-annotations)
- [Phase 6: Form Fields](#phase-6-form-fields)
- [Phase 7: Advanced Features](#phase-7-advanced-features)

---

## Phase 3: Universal API (Pdf)

### Creating PDFs from Various Sources

```java
import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfBuilder;
import com.pdfoxide.creation.PageSize;

// Create from Markdown
Pdf doc = Pdf.fromMarkdown("# Title\n\nContent");
doc.save("from_markdown.pdf");
doc.close();

// Create from HTML
Pdf doc = Pdf.fromHtml("<h1>Title</h1><p>Content</p>");
doc.save("from_html.pdf");
doc.close();

// Create from Plain Text
Pdf doc = Pdf.fromText("Title\n\nContent");
doc.save("from_text.pdf");
doc.close();

// Create from Image
Pdf doc = Pdf.fromImage("image.png");
doc.save("from_image.pdf");
doc.close();

// Create with PdfBuilder for customization
Pdf doc = PdfBuilder.create()
    .title("My Document")
    .author("John Doe")
    .subject("PDF Oxide Example")
    .keywords("pdf", "oxide", "java")
    .pageSize(PageSize.A4)
    .margins(72.0, 72.0, 72.0, 72.0)  // 1 inch
    .fromMarkdown("# Title\n\nContent");

doc.save("custom.pdf");
doc.close();
```

### Opening and Reading PDFs

```java
import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfDocument;

// Open for reading
try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    System.out.println("Pages: " + doc.getPageCount());
    System.out.println("Version: " + doc.getVersion()[0] + "." + doc.getVersion()[1]);
    System.out.println("Has structure: " + doc.hasStructureTree());
}

// Open for editing
Pdf pdf = Pdf.open("document.pdf");
pdf.save("modified.pdf");
pdf.close();
```

### Converting Document Formats

```java
import com.pdfoxide.conversion.ConversionOptions;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    ConversionOptions opts = ConversionOptions.builder()
        .detectHeadings(true)
        .preserveLayout(false)
        .build();

    // To Markdown
    String markdown = doc.toMarkdown(0, opts);

    // To HTML
    String html = doc.toHtml(0, opts);

    // To Plain Text
    String text = doc.toPlainText(0, opts);

    // All pages
    String allMarkdown = doc.toMarkdownAll(opts);
}
```

---

## Phase 4: DOM Navigation

### Navigating Document Structure

```java
import com.pdfoxide.core.Pdf;
import com.pdfoxide.dom.PdfPage;
import com.pdfoxide.dom.PdfText;
import com.pdfoxide.dom.PdfImage;

Pdf doc = Pdf.open("document.pdf");

// Access page
int pageCount = doc.getPageCount();  // Get total pages
PdfPage page = doc.getPage(0);       // Get first page

// Find text elements
List<PdfText> allText = page.findTextContaining("");
List<PdfText> search = page.findTextContaining("example");

// Find images
List<PdfImage> images = page.findImages();

// Find paths/vectors
List<PdfPath> paths = page.findPaths();

// Find tables
List<PdfTable> tables = page.findTables();

doc.close();
```

### Modifying Page Content

```java
import com.pdfoxide.core.Pdf;
import com.pdfoxide.dom.PdfPage;
import com.pdfoxide.dom.PdfText;

Pdf doc = Pdf.open("document.pdf");
PdfPage page = doc.getPage(0);

// Find and modify text
List<PdfText> texts = page.findTextContaining("old");
for (PdfText text : texts) {
    page.setText(text.getId(), "new");
}

// Add new text
TextContent content = TextContent.builder()
    .text("Added text")
    .position(100.0, 700.0)
    .fontSize(12.0)
    .fontName("Helvetica")
    .color(0.0, 0.0, 0.0)
    .build();
ElementId id = page.addText(content);

// Add image
ImageContent image = ImageContent.builder()
    .position(100.0, 600.0)
    .width(200.0)
    .height(150.0)
    .path("image.png")
    .build();
page.addImage(image);

// Save changes
doc.savePage(page);
doc.save("modified.pdf");
doc.close();
```

---

## Phase 5: Annotations

### Adding Annotations

```java
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.annotations.*;
import com.pdfoxide.geometry.Rect;

DocumentEditor editor = DocumentEditor.open("document.pdf");

// Text annotation (comment/sticky note)
TextAnnotation comment = TextAnnotationBuilder.create(
    new Rect(100, 700, 150, 20),
    "This is a comment"
)
    .author("John Doe")
    .color(1.0, 1.0, 0.0)  // Yellow
    .build();

// Highlight annotation
HighlightAnnotation highlight = HighlightAnnotationBuilder.create(
    new Rect(200, 650, 300, 30)
)
    .color(1.0, 1.0, 0.0)  // Yellow
    .build();

// Link annotation
LinkAnnotation link = LinkAnnotationBuilder.create(
    new Rect(400, 600, 500, 30),
    LinkAction.externalLink("https://example.com")
)
    .build();

// Stamp annotation
StampAnnotation stamp = StampAnnotationBuilder.create(
    new Rect(100, 500, 150, 30),
    StampType.APPROVED
)
    .color(0.0, 1.0, 0.0)  // Green
    .build();

// Watermark annotation
WatermarkAnnotation watermark = WatermarkAnnotationBuilder.create(
    new Rect(50, 300, 700, 600),
    "CONFIDENTIAL"
)
    .opacity(0.3)
    .build();

editor.addAnnotation(0, comment);
editor.addAnnotation(0, highlight);
editor.addAnnotation(0, link);
editor.addAnnotation(0, stamp);
editor.addAnnotation(0, watermark);

editor.save("annotated.pdf");
editor.close();
```

---

## Phase 6: Form Fields

### Creating Form Fields

```java
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.forms.*;
import com.pdfoxide.geometry.Rect;

DocumentEditor editor = DocumentEditor.open("document.pdf");

// Text field
TextField nameField = TextFieldBuilder.create(
    "name",
    new Rect(100, 700, 300, 20)
)
    .defaultValue("John Doe")
    .maxLength(100)
    .required(true)
    .build();

// Checkbox field
CheckboxField subscribe = CheckboxBuilder.create(
    "subscribe",
    new Rect(100, 670, 15, 15)
)
    .defaultChecked(false)
    .exportValue("yes")
    .build();

// Combo box field
ComboBoxField country = ComboBoxBuilder.create(
    "country",
    new Rect(100, 640, 150, 20)
)
    .options("USA", "Canada", "Mexico")
    .editable(false)
    .build();

// List box field
ListBoxField skills = ListBoxBuilder.create(
    "skills",
    new Rect(100, 600, 200, 60)
)
    .options("Java", "Python", "Rust", "Go", "JavaScript")
    .multiSelect(true)
    .build();

// Radio button group
RadioButtonGroup priority = RadioButtonGroupBuilder.create("priority")
    .addButton(new Rect(100, 550, 15, 15), "high")
    .addButton(new Rect(130, 550, 15, 15), "medium")
    .addButton(new Rect(160, 550, 15, 15), "low")
    .defaultValue("medium")
    .build();

// Push button
PushButtonField submit = PushButtonBuilder.create(
    "submit",
    new Rect(100, 500, 100, 30),
    "Submit"
)
    .action(ButtonAction.submit("http://example.com/submit"))
    .build();

// Add fields
editor.addFormField(0, nameField);
editor.addFormField(0, subscribe);
editor.addFormField(0, country);
editor.addFormField(0, skills);
editor.addFormField(0, submit);

// Set field values
editor.setFormFieldValue("name", FormFieldValue.text("Jane Smith"));
editor.setFormFieldValue("subscribe", FormFieldValue.bool(true));
editor.setFormFieldValue("country", FormFieldValue.name("Canada"));

editor.save("form.pdf");
editor.close();
```

### Extracting and Exporting Form Fields

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.forms.FormExtractor;

PdfDocument doc = PdfDocument.open("form.pdf");
FormExtractor extractor = new FormExtractor(doc);

// Extract all fields
List<FormField> fields = extractor.extractFields();

for (FormField field : fields) {
    System.out.println("Field: " + field.getName());
    System.out.println("  Type: " + field.getFieldType());
    System.out.println("  Value: " + field.getValue());
    System.out.println("  Tooltip: " + field.getTooltip().orElse("N/A"));
}

// Export form data
extractor.exportFdf("data.fdf");    // FDF format
extractor.exportXfdf("data.xfdf");  // XFDF format

extractor.close();
doc.close();
```

---

## Phase 7: Advanced Features

### Text Search

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.search.TextSearcher;
import com.pdfoxide.search.SearchOptions;
import com.pdfoxide.search.SearchResult;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    TextSearcher searcher = new TextSearcher(doc);

    // Literal search
    SearchOptions opts = SearchOptions.builder()
        .caseSensitive(false)
        .wholeWord(false)
        .build();

    List<SearchResult> results = searcher.search("example", opts);

    for (SearchResult result : results) {
        System.out.printf("Found on page %d: \"%s\"%n",
            result.getPage() + 1,
            result.getText());
    }

    searcher.close();
}
```

### PDF/A Compliance Validation

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.compliance.*;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    // Validate against PDF/A-1b
    PdfAValidator validator = new PdfAValidator(PdfALevel.LEVEL_1B);
    ValidationResult result = validator.validate(doc);

    if (result.isValid()) {
        System.out.println("✓ Document is PDF/A-1b compliant");
    } else {
        System.out.println("✗ Compliance issues found:");

        for (ComplianceError error : result.getErrors()) {
            System.out.printf("ERROR: %s - %s%n",
                error.getCode(),
                error.getMessage());
        }

        for (ComplianceWarning warning : result.getWarnings()) {
            System.out.printf("WARNING: %s (Severity %d)%n",
                warning.getCode(),
                warning.getSeverity());
        }
    }

    // Print statistics
    ValidationStats stats = result.getStats();
    System.out.printf("Pages checked: %d%n", stats.getPagesChecked());
    System.out.printf("Elements analyzed: %d%n", stats.getElementsAnalyzed());
    System.out.printf("Validation time: %d ms%n", stats.getValidationTime());
}
```

### Digital Signatures (v0.3.0 Foundation)

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.security.DigitalSignature;
import com.pdfoxide.security.SignatureConfig;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    // Check for signatures
    int signatureCount = DigitalSignature.getSignatureCount(doc);
    System.out.printf("Document has %d signature(s)%n", signatureCount);

    // Get signature information (v0.3.0: foundation only)
    for (int i = 0; i < signatureCount; i++) {
        DigitalSignature sig = DigitalSignature.getSignature(doc, i);
        System.out.printf("Signature %d: %s%n", i + 1, sig.getName());
    }
}

// Adding signatures (v0.4.0+ full implementation)
// SignatureConfig config = SignatureConfig.builder()
//     .certificate(cert_bytes)
//     .privateKey(key_bytes)
//     .reason("Approved for release")
//     .location("New York")
//     .build();
```

---

## Error Handling

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.*;

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

---

## Resource Management

```java
import com.pdfoxide.core.PdfDocument;

// Pattern 1: Try-with-resources (recommended)
try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    // Use document
} // Automatically closed

// Pattern 2: Manual management
PdfDocument doc = PdfDocument.open("document.pdf");
try {
    // Use document
} finally {
    doc.close();
}

// Pattern 3: Auto-cleanup via Cleaner
// Even without close(), resources are cleaned up on GC
PdfDocument doc = PdfDocument.open("document.pdf");
// Resources cleanup guaranteed even without explicit close()
```

---

## Complete Workflow Example

```java
import com.pdfoxide.core.*;
import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.search.SearchOptions;
import com.pdfoxide.search.TextSearcher;
import java.util.List;

public class CompleteWorkflow {
    public static void main(String[] args) throws Exception {
        // 1. Create PDF from Markdown
        Pdf doc = Pdf.fromMarkdown("# Report\n\nData analysis results...");
        doc.save("report.pdf");
        doc.close();

        // 2. Read and extract
        try (PdfDocument readDoc = PdfDocument.open("report.pdf")) {
            System.out.println("Pages: " + readDoc.getPageCount());
            String text = readDoc.extractText(0);
            System.out.println("Text: " + text.substring(0, 50) + "...");
        }

        // 3. Edit document
        doc = Pdf.open("report.pdf");
        // Modify content...
        doc.save("report_edited.pdf");
        doc.close();

        // 4. Search
        try (TextSearcher searcher = new TextSearcher(
                PdfDocument.open("report_edited.pdf"))) {
            List<SearchResult> results = searcher.search(
                "analysis",
                SearchOptions.builder().caseSensitive(false).build()
            );
            System.out.println("Found " + results.size() + " matches");
        }
    }
}
```

---

## Performance Tips

1. **Batch Operations**: Process multiple documents efficiently
2. **Resource Cleanup**: Always use try-with-resources
3. **Memory**: For large PDFs, process page-by-page
4. **Caching**: Reuse document handles when possible

---

**Last Updated**: January 15, 2026
**Version**: 0.3.0
