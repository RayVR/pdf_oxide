# PDF Oxide Java Bindings - Getting Started

High-performance PDF processing for Java with complete API feature parity to Rust.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Examples](#examples)
- [Documentation](#documentation)

## Features

### Core Capabilities
- **Read**: Extract text, metadata, and structure from PDF documents
- **Create**: Generate PDFs from Markdown, HTML, text, or images
- **Edit**: Modify PDF content with DOM-like navigation and manipulation
- **Search**: Full-text search with regex, case-sensitivity, and whole-word options
- **Forms**: Create, extract, and manage AcroForm and XFA form fields
- **Annotations**: Add 20+ annotation types (highlights, comments, watermarks, etc.)
- **Compliance**: Validate PDF/A conformance levels (1A, 1B, 2A, 2B, 2U, 3A, 3B, 3U)
- **Signatures**: Digital signature metadata foundation (v0.3.0), full support v0.4.0+

### Conversion Formats
- Markdown (with heading detection and reading order)
- HTML (preserving layout and styling)
- Plain Text (with automatic reading order)

## Requirements

- **Java**: JDK 8 or later
- **Operating Systems**:
  - Linux (x86_64, aarch64)
  - macOS (x86_64, aarch64)
  - Windows (x86_64, aarch64)

## Installation

### Maven

Add to your `pom.xml`:

```xml
<dependency>
    <groupId>com.pdfoxide</groupId>
    <artifactId>pdf-oxide</artifactId>
    <version>0.3.0</version>
</dependency>
```

The JAR includes platform-specific native libraries that are automatically loaded for your OS.

### Gradle

Add to your `build.gradle`:

```gradle
dependencies {
    implementation 'com.pdfoxide:pdf-oxide:0.3.0'
}
```

## Quick Start

### 1. Extract Text from PDF

```java
import com.pdfoxide.core.PdfDocument;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    System.out.println("Pages: " + doc.getPageCount());
    System.out.println("Version: " + doc.getVersion()[0] + "." + doc.getVersion()[1]);

    String text = doc.extractText(0);
    System.out.println(text);
}
```

### 2. Convert PDF to Markdown

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.conversion.ConversionOptions;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    ConversionOptions opts = ConversionOptions.builder()
        .detectHeadings(true)
        .preserveLayout(false)
        .build();

    String markdown = doc.toMarkdown(0, opts);
    System.out.println(markdown);
}
```

### 3. Create PDF from Markdown

```java
import com.pdfoxide.core.Pdf;

Pdf doc = Pdf.fromMarkdown("# My Document\n\nHello, world!");
doc.save("output.pdf");
doc.close();
```

### 4. Search Text

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.search.TextSearcher;
import com.pdfoxide.search.SearchOptions;
import com.pdfoxide.search.SearchResult;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    TextSearcher searcher = new TextSearcher(doc);

    SearchOptions options = SearchOptions.builder()
        .caseSensitive(false)
        .wholeWord(true)
        .build();

    List<SearchResult> results = searcher.search("example", options);
    for (SearchResult result : results) {
        System.out.println("Found on page " + result.getPage() + ": " + result.getText());
    }

    searcher.close();
}
```

### 5. Create Form Fields

```java
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.forms.TextField;
import com.pdfoxide.geometry.Rect;

try (DocumentEditor editor = DocumentEditor.open("document.pdf")) {
    // Add text field
    TextField field = new TextField(
        "username",
        new Rect(100, 700, 150, 20),
        "Enter username",
        50  // max length
    );
    editor.addFormField(0, field);

    // Export form data
    editor.exportFormDataXfdf("form_data.xfdf");

    // Save PDF
    editor.save("form_output.pdf");
}
```

### 6. Validate PDF/A Compliance

```java
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.compliance.PdfAValidator;
import com.pdfoxide.compliance.PdfALevel;
import com.pdfoxide.compliance.ValidationResult;

try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    PdfAValidator validator = new PdfAValidator(PdfALevel.LEVEL_1B);
    ValidationResult result = validator.validate(doc);

    if (result.isValid()) {
        System.out.println("PDF/A compliant!");
    } else {
        System.out.println("Errors: " + result.getErrors().size());
        for (var error : result.getErrors()) {
            System.out.println("  - " + error.getMessage());
        }
    }
}
```

## Examples

Complete examples are available in the `examples/` directory:

- `ReadPdf.java` - Extract text and metadata
- `CreatePdf.java` - Create PDFs from multiple sources
- `EditPdf.java` - Modify PDF content
- `FormHandling.java` - Create and manage forms
- `SearchPdf.java` - Text search operations
- `ValidatePdfa.java` - PDF/A compliance checking

Run examples:

```bash
cd examples
javac -cp ../target/pdf-oxide-0.3.0.jar ReadPdf.java
java -cp .:../target/pdf-oxide-0.3.0.jar ReadPdf document.pdf
```

## Documentation

### API Overview

The Java bindings provide complete feature parity with the Rust API:

| Phase | Feature | Classes |
|-------|---------|---------|
| 3 | Universal API | Pdf, PdfBuilder |
| 4 | DOM Navigation | PdfPage, PdfElement (sealed) |
| 5 | Annotations | TextAnnotation, HighlightAnnotation, etc. (20+ types) |
| 6 | Form Fields | TextField, CheckboxField, ComboBoxField, etc. |
| 7 | Advanced | TextSearcher, PdfAValidator, DigitalSignature |

### Exception Handling

All API methods throw `PdfException` (or subclasses) for error conditions:

```java
import com.pdfoxide.exceptions.*;

try {
    PdfDocument doc = PdfDocument.open("invalid.pdf");
} catch (ParseException e) {
    System.err.println("Invalid PDF: " + e.getMessage());
} catch (EncryptionException e) {
    System.err.println("Encrypted PDF: " + e.getMessage());
} catch (PdfException e) {
    System.err.println("PDF error: " + e.getMessage());
}
```

### Resource Management

Use try-with-resources for automatic cleanup:

```java
try (PdfDocument doc = PdfDocument.open("document.pdf")) {
    // Use document
    // Automatically closed
}
```

The library uses Java 9+ Cleaner API for guaranteed cleanup even if close() isn't called.

### Memory Management

The library efficiently manages native Rust resources:

- Document pointers are wrapped in `NativeHandle`
- Cleaner API ensures resource cleanup on garbage collection
- No memory leaks even with exceptions

### Feature Detection

Check for optional features at runtime:

```java
import com.pdfoxide.util.FeatureDetection;

if (FeatureDetection.hasOcr()) {
    // Use OCR features
}

if (FeatureDetection.hasRendering()) {
    // Use rendering features
}

if (FeatureDetection.hasSignatures()) {
    // Use signature features
}
```

## Performance

The Java bindings maintain <10% overhead compared to Rust API while providing idiomatic Java interfaces.

## Building from Source

### Prerequisites

- Rust 1.70+ with `cargo`
- JDK 8+
- Maven 3.6+

### Build Steps

```bash
# Build native library
./scripts/build-natives.sh --current --release

# Build Java bindings
cd java
mvn clean verify

# Run tests
mvn test

# Package JAR
mvn package
```

## Troubleshooting

### UnsatisfiedLinkError

The native library couldn't be loaded for your platform.

**Solution**: Ensure you have the correct native library for your OS:
- Linux: `libpdf_oxide_jni.so`
- macOS: `libpdf_oxide_jni.dylib`
- Windows: `pdf_oxide_jni.dll`

### OutOfMemoryError

Processing large PDFs or many documents without closing.

**Solution**: Always use try-with-resources or explicitly call close():

```java
try (PdfDocument doc = PdfDocument.open("large.pdf")) {
    // ...
}
```

### Incorrect Platform Detection

The library couldn't determine your platform.

**Solution**: Set the system property explicitly:

```bash
java -Dos.arch=x86_64 -Dos.name=Linux MyApp.jar
```

## License

Dual licensed under MIT or Apache 2.0.

## Support

- GitHub Issues: https://github.com/yfedoseev/pdf_oxide/issues
- Documentation: https://github.com/yfedoseev/pdf_oxide/wiki
- Examples: https://github.com/yfedoseev/pdf_oxide/tree/main/java/examples

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.
