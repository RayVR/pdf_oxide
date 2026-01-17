# PDF Oxide - Node.js/TypeScript Bindings

Complete Node.js/TypeScript bindings for [pdf_oxide](https://github.com/yfedoseev/pdf_oxide) - A production-ready Rust PDF processing library.

## Features

### Core Operations
- **Text Extraction**: Extract text with automatic reading order detection (70-80% character recovery)
- **PDF Creation**: Create PDFs from Markdown, HTML, or plain text
- **PDF Editing**: Edit existing PDFs with DOM-like navigation and modification
- **Format Conversion**: Convert PDFs to Markdown, HTML, or plain text

### Advanced Features
- **Annotations**: Create and manage **27 PDF annotation types** (text, links, shapes, stamps, watermarks, 3D, etc.)
- **Full-Text Search**: Search across PDF pages with pattern matching and result positioning
- **Forms**: Extract and fill **AcroForm** (traditional) and **XFA** (XML Forms Architecture) forms
- **Metadata**: Read and write **XMP metadata** with 18 standard fields, document information, copyright
- **Page Labels**: Intelligent page numbering (decimal, Roman numerals, letters, custom prefixes)
- **Embedded Files**: Manage attached files with MIME types and metadata
- **Compliance**: PDF/A compliance validation
- **Multi-language Support**: Handles CJK, RTL, complex scripts
- **Cross-Platform**: Works on Windows, Linux, and macOS (x64 and ARM64)

## Installation

```bash
npm install @pdf-oxide/core
```

This automatically installs the correct native binary for your platform.

## Quick Start

### Reading and Text Extraction

```javascript
import { PdfDocument } from '@pdf-oxide/core';

using doc = PdfDocument.open('document.pdf');

console.log(`Pages: ${doc.pageCount}`);
const text = doc.extractText(0);
const markdown = doc.toMarkdown(0);
```

### Creating PDFs

```javascript
import { Pdf, PdfBuilder } from '@pdf-oxide/core';

// Simple creation
using doc = Pdf.fromMarkdown('# Hello\n\nWorld');
doc.save('output.pdf');

// Advanced with builder
using doc2 = PdfBuilder.create()
  .title('My Document')
  .author('John Doe')
  .fromMarkdown('# Content here');
await doc2.saveAsync('output.pdf');
```

### Editing PDFs

```javascript
import { Pdf } from '@pdf-oxide/core';

using doc = Pdf.open('input.pdf');
const page = doc.page(0);

// Find and modify text
const texts = page.findTextContaining('old');
for (const text of texts) {
  page.setText(text.id(), 'new');
}

doc.savePage(page);
doc.save('output.pdf');
```

## API Documentation

### Core Classes

#### PdfDocument (Read Interface)

Read-only PDF document access for text extraction and conversion.

```javascript
// Static methods
PdfDocument.open(path: string): PdfDocument
PdfDocument.openWithPassword(path: string, password: string): PdfDocument

// Properties
doc.version: { major: number; minor: number }
doc.pageCount: number
doc.hasStructureTree: boolean

// Methods
doc.extractText(pageIndex: number): string
doc.extractTextAsync(pageIndex: number): Promise<string>
doc.toMarkdown(pageIndex: number, options?: ConversionOptions): string
doc.toMarkdownAsync(pageIndex: number, options?: ConversionOptions): Promise<string>
doc.toMarkdownAll(options?: ConversionOptions): string
doc.toHtml(pageIndex: number, options?: ConversionOptions): string
doc.toHtmlAll(options?: ConversionOptions): string
doc.close(): void
```

#### Pdf (Create & Edit Interface)

Unified interface for creating new PDFs and editing existing ones.

```javascript
// Static factory methods
Pdf.fromMarkdown(markdown: string, config?: PdfConfig): Pdf
Pdf.fromHtml(html: string, config?: PdfConfig): Pdf
Pdf.fromText(text: string, config?: PdfConfig): Pdf
Pdf.open(path: string): Pdf

// Properties
doc.pageCount: number
doc.version: { major: number; minor: number }

// Methods
doc.page(index: number): PdfPage
doc.savePage(page: PdfPage): void
doc.save(path: string): void
doc.saveAsync(path: string): Promise<void>
doc.close(): void
```

#### PdfBuilder (Universal Interface)

Fluent API for advanced PDF creation with configuration.

```javascript
PdfBuilder.create(): PdfBuilder
  .title(title: string): PdfBuilder
  .author(author: string): PdfBuilder
  .subject(subject: string): PdfBuilder
  .pageSize(size: PageSize): PdfBuilder
  .margins(top: number, right: number, bottom: number, left: number): PdfBuilder
  .fromMarkdown(markdown: string): Pdf
  .fromHtml(html: string): Pdf
  .fromText(text: string): Pdf
```

#### PdfPage (DOM Navigation)

DOM-like access to PDF page elements.

```javascript
// Properties
page.pageIndex: number
page.width: number
page.height: number

// Methods
page.children(): PdfElement[]
page.findTextContaining(query: string): PdfText[]
page.findText(query: string, options?: SearchOptions): SearchResult[]
page.setText(elementId: string, newText: string): void
page.addElement(element: ElementContent): string
page.removeElement(elementId: string): void
page.annotations(): Annotation[]
page.addAnnotation(annotation: AnnotationContent): string
page.close(): void
```

### Types

#### Geometry Types

```typescript
interface Rect {
  x: number;      // Left position
  y: number;      // Top position
  width: number;  // Width
  height: number; // Height
}

interface Point {
  x: number;
  y: number;
}

interface Color {
  r: number;  // 0-255
  g: number;  // 0-255
  b: number;  // 0-255
  a: number;  // 0-255 (alpha)
}
```

#### Options

```typescript
interface ConversionOptions {
  detectHeadings?: boolean;
  preserveLayout?: boolean;
  includeImages?: boolean;
  imageOutputDir?: string;
  embedImages?: boolean;
}

interface SearchOptions {
  caseSensitive?: boolean;
  wholeWords?: boolean;
  regex?: boolean;
}

interface PdfConfig {
  title?: string;
  author?: string;
  subject?: string;
  pageSize?: PageSize;
  marginTop?: number;
  marginRight?: number;
  marginBottom?: number;
  marginLeft?: number;
}
```

### Error Types

All errors extend the base `PdfError` class:

- **PdfIoError** - File I/O errors (file not found, permission denied)
- **PdfParseError** - PDF format errors (invalid structure, corrupted data)
- **PdfEncryptionError** - Password incorrect, encryption errors
- **PdfUnsupportedError** - Unsupported PDF version or feature
- **PdfInvalidStateError** - Invalid operation on document
- **PdfDecodeError** - Stream decompression errors
- **PdfEncodeError** - Data encoding errors
- **PdfFontError** - Font-related errors
- **PdfImageError** - Image processing errors
- **PdfCircularReferenceError** - Circular reference in PDF structure
- **PdfRecursionLimitError** - Recursion depth exceeded
- **PdfOcrError** - OCR processing errors (optional feature)
- **PdfMlError** - Machine learning errors (optional feature)

## Examples

### Working with Forms

```javascript
import { Pdf, AcroForm } from '@pdf-oxide/core';

const doc = Pdf.fromText('Application Form');
const form = AcroForm.new('AppForm');

// Add form fields
form.addField({
  id: 'name',
  field_name: 'applicant_name',
  field_type: 'Text',
  label: 'Full Name',
  rect: { x: 50, y: 700, width: 300, height: 25 },
  page_index: 0,
  read_only: false,
  required: true,
  hidden: false,
});

form.addField({
  id: 'agree',
  field_name: 'agree_terms',
  field_type: 'Checkbox',
  label: 'I agree to the terms',
  rect: { x: 50, y: 650, width: 20, height: 20 },
  page_index: 0,
  required: true,
});

// Set field values
form.setFieldValue('applicant_name', 'John Doe');

// Apply form to document
doc.setForms(form);
doc.save('form.pdf');
```

### Working with Metadata

```javascript
import { Pdf, XMPMetadata } from '@pdf-oxide/core';

const doc = Pdf.fromMarkdown('# Report\n\nContent');

// Set document metadata
const metadata = XMPMetadata.new();
metadata.setTitle('Quarterly Report Q4 2024');
metadata.setAuthor('Finance Department');
metadata.setSubject('FY2024 Results');
metadata.setKeywords('quarterly, financial, report');
metadata.setCopyright('Copyright 2024 Acme Inc.');
metadata.setLanguage('en-US');

doc.setMetadata(metadata);
doc.save('report.pdf');

// Read metadata
const info = doc.getDocumentInfo();
console.log(`Title: ${info.title}`);
console.log(`Version: ${info.version}`);
```

### Working with Annotations

```javascript
import { Pdf } from '@pdf-oxide/core';

const doc = Pdf.fromMarkdown('# Important Document\n\nRead carefully.');
const page = doc.page(0);

// Add text annotation (sticky note)
const annotation = {
  id: 'note1',
  rect: { x: 200, y: 700, width: 50, height: 50 },
  contents: 'Please review this section',
  author: 'Jane Smith',
  subject: 'Review',
  icon_name: 'Comment',
  color_r: 255,
  color_g: 255,
  color_b: 0,
};

page.addAnnotation(annotation);
doc.savePage(page);
doc.save('annotated.pdf');
```

### Text Extraction with Async

```javascript
async function extractAllPages(pdfPath) {
  using doc = PdfDocument.open(pdfPath);

  const pages = [];
  for (let i = 0; i < doc.pageCount; i++) {
    const text = await doc.extractTextAsync(i);
    pages.push(text);
  }
  return pages;
}

const pages = await extractAllPages('document.pdf');
console.log(`Extracted ${pages.length} pages`);
```

### DOM-like Editing

```javascript
import { Pdf } from '@pdf-oxide/core';

using doc = Pdf.open('input.pdf');
const page = doc.page(0);

// Navigate DOM tree
for (const element of page.children()) {
  if (element.isText()) {
    console.log('Text:', element.asText().text());
  } else if (element.isImage()) {
    console.log('Image:', element.asImage().width(), 'x', element.asImage().height());
  }
}

doc.savePage(page);
doc.save('output.pdf');
```

### Full-Text Search

```javascript
import { Pdf } from '@pdf-oxide/core';

using doc = Pdf.open('document.pdf');
const page = doc.page(0);

const results = page.findText('example', { caseSensitive: true });

const largeMatches = results
  .filter(r => r.bbox.width > 100)
  .sort((a, b) => a.bbox.y - b.bbox.y);

for (const match of largeMatches) {
  console.log(`Match: "${match.text}" at ${JSON.stringify(match.bbox)}`);
}
```

### Error Handling

```javascript
import {
  PdfDocument,
  PdfIoError,
  PdfParseError,
  PdfEncryptionError
} from '@pdf-oxide/core';

try {
  using doc = PdfDocument.open('document.pdf');
  const text = doc.extractText(0);
} catch (err) {
  if (err instanceof PdfEncryptionError) {
    console.error('PDF is encrypted. Use openWithPassword()');
  } else if (err instanceof PdfIoError) {
    console.error('Cannot open file:', err.message);
  } else if (err instanceof PdfParseError) {
    console.error('Invalid PDF:', err.message);
  }
}
```

## Resource Management

Use the explicit resource management pattern (with `using`) to ensure cleanup:

```javascript
// Recommended (when available)
using doc = PdfDocument.open('file.pdf');
const text = doc.extractText(0);
// Automatically cleaned up

// Alternative (try/finally)
let doc;
try {
  doc = PdfDocument.open('file.pdf');
  const text = doc.extractText(0);
} finally {
  if (doc) doc.close();
}
```

## Performance

- **Zero-cost abstraction**: Native module overhead <5% vs raw Rust
- **Async I/O**: Non-blocking PDF operations with async/await
- **Memory efficient**: Streaming processing for large PDFs
- **Parallel processing**: Multi-threaded extraction

Benchmarks show **47.9× faster** text extraction than PyMuPDF4LLM.

## Platform Support

| Platform | x64 | ARM64 |
|----------|-----|-------|
| Windows  | ✓   | ✓     |
| Linux    | ✓   | ✓     |
| macOS    | ✓   | ✓     |

## Building from Source

```bash
cd nodejs
npm install
npm run build

# Test
npm test

# Examples
node examples/read-extract.js document.pdf
node examples/create-pdf.js
node examples/error-handling.js
```

## Feature Flags

Build with optional features:

```bash
# With rendering support
CARGO_BUILD_FLAGS="--features rendering" npm run build

# With OCR support
CARGO_BUILD_FLAGS="--features ocr" npm run build

# All features
CARGO_BUILD_FLAGS="--features all" npm run build
```

## TypeScript Support

Full TypeScript definitions are auto-generated from Rust:

```typescript
import { PdfDocument, ConversionOptions } from '@pdf-oxide/core';

const doc = PdfDocument.open('document.pdf');
const options: ConversionOptions = {
  detectHeadings: true,
  preserveLayout: false,
  includeImages: true,
  embedImages: true,
};
const markdown: string = doc.toMarkdown(0, options);
```

## API Stability

The API is production-ready and stable:
- Semantic versioning (MAJOR.MINOR.PATCH)
- Breaking changes only in major versions
- Deprecation notices in minor versions

## License

Dual licensed under MIT OR Apache-2.0

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../CONTRIBUTING.md)

## Acknowledgments

Built on top of the excellent pdf_oxide Rust library.
