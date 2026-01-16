# PDF Oxide Java Examples

This directory contains practical examples demonstrating all features of the PDF Oxide Java bindings.

## Examples Overview

### 1. ReadPdf.java
**What it does**: Opens a PDF and extracts text and metadata

**Demonstrates**:
- Opening PDF documents
- Reading document properties (version, page count)
- Extracting text content
- Converting to Markdown
- Converting to HTML

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar ReadPdf.java
java -cp .:../target/pdf-oxide-*.jar ReadPdf your_document.pdf
```

**Output**:
- Document information
- Text extraction from first page
- Markdown and HTML conversions

---

### 2. CreatePdf.java
**What it does**: Creates PDF documents from multiple sources

**Demonstrates**:
- Creating PDF from Markdown
- Creating PDF from HTML
- Creating PDF from plain text
- Using PdfBuilder for customization
- Setting document metadata (title, author, etc.)
- Configuring page size and margins

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar CreatePdf.java
java -cp .:../target/pdf-oxide-*.jar CreatePdf
```

**Output**: Creates four PDF files
- `output_markdown.pdf` - From Markdown source
- `output_html.pdf` - From HTML source
- `output_text.pdf` - From plain text
- `output_builder.pdf` - Using PdfBuilder with custom settings

---

### 3. SearchPdf.java
**What it does**: Demonstrates full-text search capabilities

**Demonstrates**:
- Creating searchable PDFs
- Literal text search
- Case-sensitive search
- Whole-word search
- Page-specific search
- Search result retrieval

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar SearchPdf.java
java -cp .:../target/pdf-oxide-*.jar SearchPdf
```

**Output**:
- Search results with page numbers and context
- Comparison of different search options
- Statistics on search performance

---

### 4. ValidatePdfa.java
**What it does**: Validates PDF/A compliance

**Demonstrates**:
- PDF/A compliance validation
- Testing against different PDF/A levels (1B, 2B, 3B)
- Analyzing validation errors and warnings
- Understanding compliance requirements
- Generating compliance reports

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar ValidatePdfa.java
java -cp .:../target/pdf-oxide-*.jar ValidatePdfa
```

**Output**:
- Compliance status for each PDF/A level
- List of errors and warnings
- Validation statistics

---

### 5. EditPdf.java
**What it does**: Opens and modifies PDF documents

**Demonstrates**:
- Opening PDF for editing
- DOM navigation
- Finding text elements
- Modifying document content
- Saving modified PDFs

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar EditPdf.java
java -cp .:../target/pdf-oxide-*.jar EditPdf
```

**Output**: Creates modified PDF
- `sample_edit.pdf` - Original document
- `sample_edit_modified.pdf` - Modified version

---

### 6. FormHandling.java
**What it does**: Works with form fields

**Demonstrates**:
- Creating documents with form fields
- Adding text fields to PDFs
- Extracting form field information
- Exporting form data (FDF/XFDF)
- Field value management

**Run it**:
```bash
javac -cp ../target/pdf-oxide-*.jar FormHandling.java
java -cp .:../target/pdf-oxide-*.jar FormHandling
```

**Output**: Creates form documents
- `sample_form.pdf` - Basic form layout
- `sample_form_with_fields.pdf` - Form with fields
- `form_data.fdf` - Exported form data (FDF format)
- `form_data.xfdf` - Exported form data (XFDF format)

---

## Prerequisites

### Build the JAR
First, ensure you have built the PDF Oxide JAR:

```bash
cd /home/yfedoseev/projects/pdf_oxide
./scripts/build-natives.sh --current --release
cd java
mvn clean package
```

This creates `java/target/pdf-oxide-*.jar` with all native libraries.

### JDK
You need JDK 8 or later:

```bash
javac -version
java -version
```

---

## Running Examples

### Option 1: Direct Compilation and Execution

```bash
cd /home/yfedoseev/projects/pdf_oxide/java/examples

# Compile all examples
javac -cp ../target/pdf-oxide-*.jar *.java

# Run specific example
java -cp .:../target/pdf-oxide-*.jar ReadPdf sample.pdf
```

### Option 2: One-Liner

```bash
cd /home/yfedoseev/projects/pdf_oxide/java/examples
javac -cp ../target/pdf-oxide-*.jar ReadPdf.java && \
java -cp .:../target/pdf-oxide-*.jar ReadPdf sample.pdf
```

### Option 3: Using Maven

From the `java` directory:

```bash
# Compile examples
mvn compile

# Run example (if Maven configured)
mvn exec:java -Dexec.mainClass="com.example.ReadPdf"
```

---

## Understanding the Examples

### Key Concepts

**Try-With-Resources**:
All examples use try-with-resources for automatic resource cleanup:
```java
try (PdfDocument doc = PdfDocument.open(path)) {
    // Use document
} // Automatically closed
```

**Exception Handling**:
Examples demonstrate proper error handling:
```java
catch (PdfException e) {
    System.err.println("PDF Error: " + e.getMessage());
}
```

**Builder Pattern**:
Some examples use builders for configuration:
```java
Pdf doc = PdfBuilder.create()
    .title("Document")
    .author("John Doe")
    .fromMarkdown(markdown);
```

---

## Troubleshooting

### Problem: UnsatisfiedLinkError

**Cause**: Native library not found

**Solution**:
```bash
# Verify JAR contains natives
jar tf ../target/pdf-oxide-*.jar | grep -E "\.so|\.dylib|\.dll"

# Rebuild if needed
cd /home/yfedoseev/projects/pdf_oxide
./scripts/build-natives.sh --current --release
cd java && mvn clean package
```

### Problem: ClassNotFoundException

**Cause**: Wrong classpath or JAR not built

**Solution**:
```bash
# Check JAR exists
ls -la ../target/pdf-oxide-*.jar

# Rebuild if needed
cd ../
mvn clean package
cd examples
```

### Problem: File Not Found

**Cause**: Sample files not created yet

**Solution**: Most examples create sample files automatically. Just run them:
```bash
java -cp .:../target/pdf-oxide-*.jar CreatePdf
```

---

## Example Output

### ReadPdf Output
```
PDF Document Information
----
PDF Version: 1.7
Total Pages: 1
Tagged PDF: No

TEXT EXTRACTION (First Page)
----
<extracted text content>

MARKDOWN CONVERSION (First Page)
----
# Document Title
<markdown content>
```

### CreatePdf Output
```
1. Creating from Markdown...
   ✓ Created: output_markdown.pdf

2. Creating from HTML...
   ✓ Created: output_html.pdf

3. Creating from Text...
   ✓ Created: output_text.pdf

4. Creating with Builder (custom settings)...
   ✓ Created: output_builder.pdf
```

---

## Learning Path

Start with these examples in order:

1. **ReadPdf** - Learn how to open and read PDFs
2. **CreatePdf** - Learn how to create PDFs from various sources
3. **EditPdf** - Learn how to modify PDF content
4. **SearchPdf** - Learn how to search text
5. **ValidatePdfa** - Learn about compliance validation
6. **FormHandling** - Learn about form fields

---

## Next Steps

### Modify Examples
Try modifying these examples to:
- Use different PDF sources
- Search for different text patterns
- Validate against different PDF/A levels
- Add more form fields

### Create Your Own
Use these examples as templates for your own PDF processing tasks:

```java
// Template
try (PdfDocument doc = PdfDocument.open(path)) {
    // Your code here
}
```

### Explore the API
Review the Java bindings documentation for more classes:
- `com.pdfoxide.core` - Core APIs
- `com.pdfoxide.dom` - Document structure
- `com.pdfoxide.search` - Text search
- `com.pdfoxide.compliance` - Validation
- `com.pdfoxide.forms` - Form fields
- `com.pdfoxide.annotations` - Annotations

---

## Support

For issues or questions:
- Check the main README: `../GETTING_STARTED.md`
- Review the implementation guide: `../../PHASE_8_GUIDE.md`
- Check existing examples for patterns

---

**Last Updated**: January 15, 2026
**PDF Oxide Version**: 0.3.0
**Java Compatibility**: JDK 8+
