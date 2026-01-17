# PDF Oxide C# Bindings - Complete API Guide

A comprehensive guide to the pdf_oxide C# bindings API covering all public classes, methods, and patterns.

**Version**: 1.0 (Phases 1-4 Complete)
**Target Frameworks**: .NET Standard 2.0+, .NET 5+, .NET 6+
**Package**: PdfOxide (NuGet)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Concepts](#core-concepts)
3. [Reading PDFs](#reading-pdfs)
4. [Creating PDFs](#creating-pdfs)
5. [Editing PDFs](#editing-pdfs)
6. [Working with Elements](#working-with-elements)
7. [Working with Annotations](#working-with-annotations)
8. [Text Search](#text-search)
9. [Geometry Types](#geometry-types)
10. [Exception Handling](#exception-handling)
11. [Best Practices](#best-practices)
12. [Advanced Patterns](#advanced-patterns)

---

## Quick Start

### Installation

```bash
dotnet add package PdfOxide
```

### Your First PDF

```csharp
using PdfOxide.Core;

// Read a PDF
using (var doc = PdfDocument.Open("document.pdf"))
{
    var text = doc.ExtractText(0);
    Console.WriteLine(text);
}

// Create a PDF from Markdown
using (var pdf = Pdf.FromMarkdown("# Hello World\n\nThis is my PDF"))
{
    pdf.Save("output.pdf");
}
```

---

## Core Concepts

### IDisposable Pattern

All PDF objects implement `IDisposable` for proper resource cleanup:

```csharp
// Using statement (recommended)
using (var doc = PdfDocument.Open("file.pdf"))
{
    // Use document
} // Automatically disposed

// Manual disposal
var doc = PdfDocument.Open("file.pdf");
try
{
    // Use document
}
finally
{
    doc.Dispose();
}
```

### Error Handling

All PDF operations throw typed exceptions:

```csharp
try
{
    using (var doc = PdfDocument.Open("missing.pdf"))
    {
        // ...
    }
}
catch (PdfIoException ex)
{
    Console.WriteLine($"File error: {ex.Message}");
}
catch (PdfParseException ex)
{
    Console.WriteLine($"PDF corruption: {ex.Message}");
}
catch (PdfException ex)
{
    Console.WriteLine($"Generic PDF error: {ex.Message}");
}
```

### Async Operations

I/O operations support async/await:

```csharp
// Async extraction
var text = await doc.ExtractTextAsync(0);

// Async save
await pdf.SaveAsync("output.pdf");

// With cancellation
var cts = new CancellationTokenSource(timeout);
var text = await doc.ExtractTextAsync(0, cts.Token);
```

---

## Reading PDFs

### PdfDocument Class

Read-only access to PDF documents with text extraction and format conversion.

#### Opening PDFs

```csharp
// From file
using (var doc = PdfDocument.Open("document.pdf"))
{
    // Use document
}

// From stream
using (var stream = File.OpenRead("document.pdf"))
using (var doc = PdfDocument.Open(stream))
{
    // Use document
}

// From bytes
byte[] pdfData = File.ReadAllBytes("document.pdf");
using (var doc = PdfDocument.Open(pdfData.AsMemory()))
{
    // Use document
}

// With password
using (var doc = PdfDocument.OpenWithPassword("secure.pdf", "password"))
{
    // Use protected document
}
```

#### Document Properties

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    // Version information
    var (major, minor) = doc.Version;
    Console.WriteLine($"PDF {major}.{minor}");

    // Page count
    int pages = doc.PageCount;
    Console.WriteLine($"Total pages: {pages}");

    // Structure information
    if (doc.HasStructureTree)
    {
        Console.WriteLine("Tagged PDF with structure tree");
    }
}
```

#### Text Extraction

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    // Extract from single page
    string text = doc.ExtractText(0);

    // Extract all pages
    for (int i = 0; i < doc.PageCount; i++)
    {
        var pageText = doc.ExtractText(i);
        Console.WriteLine($"Page {i + 1}:\n{pageText}\n");
    }

    // Async extraction
    var asyncText = await doc.ExtractTextAsync(0);
}
```

#### Format Conversion

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    var options = new ConversionOptions
    {
        DetectHeadings = true,
        PreserveLayout = false,
        IncludeImages = true
    };

    // To Markdown
    string markdown = doc.ToMarkdown(0, options);
    File.WriteAllText("page.md", markdown);

    // To HTML
    string html = doc.ToHtml(0, options);
    File.WriteAllText("page.html", html);

    // To plain text
    string plainText = doc.ToText(0, options);
    File.WriteAllText("page.txt", plainText);

    // Convert all pages
    string allMarkdown = doc.ToMarkdownAll(options);
    File.WriteAllText("document.md", allMarkdown);
}
```

---

## Creating PDFs

### Pdf & PdfBuilder Classes

Create PDF documents from Markdown, HTML, or text content.

#### Simple Creation

```csharp
// From Markdown
using (var pdf = Pdf.FromMarkdown("# Title\n\nContent here"))
{
    pdf.Save("output.pdf");
}

// From HTML
using (var pdf = Pdf.FromHtml("<h1>Title</h1><p>Content</p>"))
{
    pdf.Save("output.pdf");
}

// From plain text
using (var pdf = Pdf.FromText("Hello World"))
{
    pdf.Save("output.pdf");
}

// From file
using (var pdf = Pdf.FromFile("document.md"))
{
    pdf.Save("output.pdf");
}
```

#### Builder Pattern

```csharp
using (var pdf = PdfBuilder.Create()
    .WithTitle("Quarterly Report")
    .WithAuthor("John Doe")
    .WithSubject("Q4 2024 Financial Report")
    .WithCreator("MyApp 1.0")
    .WithPageSize(PageSize.A4)
    .WithMargins(72.0, 72.0, 72.0, 72.0)
    .WithOrientation(PageOrientation.Portrait)
    .FromMarkdown(@"
# Q4 2024 Financial Report

## Executive Summary
...

## Financial Results
...
    "))
{
    pdf.Save("report.pdf");
}
```

#### Saving Options

```csharp
using (var pdf = Pdf.FromMarkdown("Content"))
{
    // Save to file
    pdf.Save("output.pdf");

    // Save to stream
    using (var stream = File.Create("output.pdf"))
    {
        pdf.Save(stream);
    }

    // Save as bytes
    byte[] pdfData = pdf.ToBytes();

    // Async save
    await pdf.SaveAsync("output.pdf");
}
```

#### Page Configuration

```csharp
using (var pdf = PdfBuilder.Create()
    .WithPageSize(PageSize.Letter)      // Letter, Legal, A4, A3, A5, etc.
    .WithOrientation(PageOrientation.Landscape)
    .WithMargins(36.0, 36.0, 36.0, 36.0)  // top, right, bottom, left
    .WithLineHeight(1.5)
    .WithFontSize(12.0)
    .FromMarkdown("Content"))
{
    pdf.Save("output.pdf");
}
```

---

## Editing PDFs

### DocumentEditor Class

Modify existing PDF documents - add/remove pages, modify content, and update metadata.

#### Opening for Edit

```csharp
// Open existing PDF for editing
using (var editor = DocumentEditor.Open("document.pdf"))
{
    // Modify document
    editor.Save("output.pdf");
}
```

#### Page Operations

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    var page = editor.GetPage(0);

    // Get page properties
    var width = page.Width;
    var height = page.Height;

    // Find text on page
    var results = page.FindText("search term");
    foreach (var result in results)
    {
        Console.WriteLine($"Found at: {result.BoundingBox}");
    }

    // Get elements
    var elements = page.FindElements();
    var textElements = elements.OfType<TextElement>().ToList();

    // Get annotations
    var annotations = page.GetAnnotations();
}
```

#### Content Modification

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    var page = editor.GetPage(0);

    // Replace text
    var results = page.FindText("old");
    foreach (var result in results)
    {
        page.ReplaceText(result, "new");
    }

    // Add text
    page.AddText(100.0, 100.0, "New text", fontSize: 12.0);

    // Add image
    page.AddImage(100.0, 200.0, "image.jpg");

    editor.Save("output.pdf");
}
```

#### Metadata Operations

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    var info = editor.DocumentInfo;

    // Read metadata
    string title = info.Title;
    string author = info.Author;
    DateTime created = info.CreationDate;

    // Update metadata
    info.Title = "New Title";
    info.Author = "New Author";
    info.Subject = "New Subject";
    info.Keywords = "new, keywords";

    editor.Save("output.pdf");
}
```

---

## Working with Elements

### Element Access

Elements represent PDF content like text, images, paths, and tables.

#### Finding Elements

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    var page = doc.GetPage(0);

    // Get all elements
    var allElements = page.FindElements();

    // Filter by type
    var textElements = allElements.OfType<TextElement>().ToList();
    var images = allElements.OfType<ImageElement>().ToList();
    var tables = allElements.OfType<TableElement>().ToList();

    // LINQ queries
    var largeText = allElements
        .OfType<TextElement>()
        .Where(e => e.FontSize > 14)
        .OrderBy(e => e.BoundingBox.Y)
        .ToList();
}
```

### TextElement

Access text content and formatting information.

```csharp
var element = allElements.OfType<TextElement>().First();

// Content access
string content = element.Content;
float fontSize = element.FontSize;
string fontName = element.FontName ?? "Unknown";

// Geometry
var bbox = element.BoundingBox;
var (x, y, width, height) = (bbox.X, bbox.Y, bbox.Width, bbox.Height);

// Center point
var center = element.Center;

// Styling (when available)
bool isBold = element.IsBold;
bool isItalic = element.IsItalic;
var color = element.Color;
```

### ImageElement

Extract image data and properties.

```csharp
var image = allElements.OfType<ImageElement>().First();

// Format and dimensions
var format = image.Format;  // Jpeg, Png, Jpeg2000, Jbig2, Raw, Unknown
var (width, height) = image.Dimensions;
float aspectRatio = image.AspectRatio;

// Extract image data
byte[] imageData = image.ImageData;
if (imageData.Length > 0)
{
    // Save to file based on format
    string extension = format switch
    {
        ImageFormat.Jpeg => ".jpg",
        ImageFormat.Png => ".png",
        ImageFormat.Jpeg2000 => ".jpx",
        ImageFormat.Jbig2 => ".jbig2",
        _ => ".bin"
    };

    File.WriteAllBytes($"image{extension}", imageData);
}

// Metadata
string altText = image.AltText ?? "";
float? dpiX = image.HorizontalDpi;
float? dpiY = image.VerticalDpi;
bool isGrayscale = image.IsGrayscale;
```

### PathElement

Vector graphics information.

```csharp
var path = allElements.OfType<PathElement>().First();

// Styling
var strokeColor = path.StrokeColor;
var fillColor = path.FillColor;
float lineWidth = path.LineWidth;
var fillMode = path.FillMode;  // None, NonZeroWinding, EvenOdd
var strokeStyle = path.StrokeStyle;  // None, Solid, Dashed, Dotted, DashDot

// Properties
bool isStroked = path.IsStroked;
bool isFilled = path.IsFilled;
```

### TableElement

Structured table data access.

```csharp
var table = allElements.OfType<TableElement>().First();

// Dimensions
int rows = table.RowCount;
int cols = table.ColumnCount;

// Cell access
string content = table.GetCellContent(row: 0, col: 0);
var cellBbox = table.GetCellBoundingBox(row: 0, col: 0);

// Row/Column access
var firstRow = table.GetRow(0);  // IReadOnlyList<string>
var firstCol = table.GetColumn(0);

// 2D array
string[,] allContent = table.GetCellContents();

// Row enumeration
var rows = table.GetRows();  // IReadOnlyList<IReadOnlyList<string>>
foreach (var rowData in rows)
{
    var cellContents = string.Join(" | ", rowData);
    Console.WriteLine(cellContents);
}
```

### StructureElement

Logical PDF structure and accessibility.

```csharp
var structure = allElements.OfType<StructureElement>().First();

// Structure information
string type = structure.StructureType;
string altText = structure.AltText ?? "";
string actualText = structure.ActualText ?? "";
bool isRemoved = structure.IsRemoved;
```

---

## Working with Annotations

### Annotation Access

Annotations are interactive elements like comments, links, and highlights.

#### Finding Annotations

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    var page = doc.GetPage(0);

    // Get all annotations
    var annotations = page.GetAnnotations();

    // Filter by type
    var textAnnotations = annotations.OfType<TextAnnotation>().ToList();
    var links = annotations.OfType<LinkAnnotation>().ToList();
    var highlights = annotations.OfType<TextMarkupAnnotation>().ToList();

    // LINQ queries
    var redAnnotations = annotations
        .Where(a => a.Color?.Red == 255)
        .ToList();
}
```

### Common Properties

All annotations share common properties:

```csharp
var annotation = annotations.First();

// Content
string contents = annotation.Contents;
string subject = annotation.Subject;
string author = annotation.Author;

// Appearance
var bbox = annotation.BoundingBox;
var color = annotation.Color;  // Color with RGBA
float opacity = annotation.Opacity;
var flags = annotation.Flags;  // AnnotationFlags enum

// Metadata
var creationDate = annotation.CreationDate;
var modificationDate = annotation.ModificationDate;
```

### TextAnnotation

Sticky notes and comments.

```csharp
var note = annotation as TextAnnotation;
if (note != null)
{
    // Icon type
    var icon = note.Icon;  // Comment, Key, Note, Help, NewParagraph, etc.

    // State
    bool isOpen = note.IsOpen;

    // Access common properties
    Console.WriteLine($"Comment: {note.Contents}");
    Console.WriteLine($"By: {note.Author}");
    Console.WriteLine($"Icon: {icon}");
}
```

### LinkAnnotation

Navigation and web links.

```csharp
var link = annotation as LinkAnnotation;
if (link != null)
{
    // Link type detection
    if (link.IsUriLink)
    {
        string uri = link.Uri;
        Console.WriteLine($"Web link: {uri}");
    }
    else if (link.IsPageLink)
    {
        int targetPage = link.DestinationPage;
        Console.WriteLine($"Link to page: {targetPage + 1}");
    }
}
```

### TextMarkupAnnotation

Highlighting and text markup.

```csharp
var markup = annotation as TextMarkupAnnotation;
if (markup != null)
{
    // Markup type
    var markupType = markup.MarkupType;
    // Highlight, Underline, StrikeOut, Squiggly

    // Type helpers
    bool isHighlight = markup.IsHighlight;
    bool isUnderline = markup.IsUnderline;
    bool isStrikeOut = markup.IsStrikeOut;
    bool isSquiggly = markup.IsSquiggly;
}
```

### FreeTextAnnotation

Text boxes and callouts.

```csharp
var textBox = annotation as FreeTextAnnotation;
if (textBox != null)
{
    string fontName = textBox.FontName;
    float fontSize = textBox.FontSize;

    Console.WriteLine($"Text box: {textBox.Contents}");
    Console.WriteLine($"Font: {fontName} {fontSize}pt");
}
```

### ShapeAnnotation

Geometric shapes.

```csharp
var shape = annotation as ShapeAnnotation;
if (shape != null)
{
    // Shape type detection
    bool isSquare = shape.IsSquare;
    bool isCircle = shape.IsCircle;
    bool isLine = shape.IsLine;
    bool isPolygon = shape.IsPolygon;
    bool isPolyline = shape.IsPolyLine;
}
```

### SpecialAnnotation

Other annotation types (stamps, watermarks, etc.).

```csharp
var special = annotation as SpecialAnnotation;
if (special != null)
{
    // Type detection
    bool isStamp = special.IsStamp;
    bool isWatermark = special.IsWatermark;
    bool isRedaction = special.IsRedact;
    bool isFileAttachment = special.IsFileAttachment;
    bool isInk = special.IsInk;

    // ... and 9 more type checkers
}
```

---

## Text Search

### Search Operations

Find and locate text within PDF pages.

#### Single Page Search

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    var page = doc.GetPage(0);

    // Case-sensitive search
    var results = page.FindText("specific term", caseSensitive: true);

    // Case-insensitive search
    var allMatches = page.FindText("term", caseSensitive: false);

    // Multi-word phrase
    var phrases = page.FindText("the quick brown fox");
}
```

#### Processing Results

```csharp
var results = page.FindText("search");

foreach (var result in results)
{
    // Content
    string matchedText = result.Text;

    // Location
    var bbox = result.BoundingBox;
    float left = result.Left;
    float top = result.Top;
    float width = result.Width;
    float height = result.Height;
    var center = result.Center;

    // Page information
    int pageIndex = result.PageIndex;

    Console.WriteLine($"Found '{matchedText}' at ({left}, {top})");
}
```

#### Advanced Queries

```csharp
// Filter results by page position
var topResults = results
    .Where(r => r.Top < 200)  // Top of page
    .ToList();

// Find first occurrence
var firstMatch = results.FirstOrDefault();

// Sort by position
var sortedByX = results
    .OrderBy(r => r.BoundingBox.X)
    .ThenBy(r => r.BoundingBox.Y)
    .ToList();

// Count matches
int matchCount = results.Count();

// Get unique matches
var uniqueMatches = results
    .Select(r => r.Text)
    .Distinct()
    .ToList();
```

#### Document-Wide Search

```csharp
using (var doc = PdfDocument.Open("document.pdf"))
{
    var allResults = new List<SearchResult>();

    for (int i = 0; i < doc.PageCount; i++)
    {
        var page = doc.GetPage(i);
        var pageResults = page.FindText("search term");
        allResults.AddRange(pageResults);
    }

    Console.WriteLine($"Found {allResults.Count} matches across document");

    // Group by page
    var byPage = allResults.GroupBy(r => r.PageIndex);
    foreach (var group in byPage)
    {
        Console.WriteLine($"Page {group.Key + 1}: {group.Count()} matches");
    }
}
```

---

## Geometry Types

### Rect Structure

Represents rectangles with position and dimensions.

```csharp
var rect = new Rect(x: 100.0f, y: 200.0f, width: 50.0f, height: 30.0f);

// Properties
float left = rect.X;
float top = rect.Y;
float width = rect.Width;
float height = rect.Height;

// Computed properties
float right = rect.Right;      // X + Width
float bottom = rect.Bottom;    // Y + Height
float area = rect.Area;        // Width * Height

// Operations
var point = new Point(125.0f, 215.0f);
bool contains = rect.Contains(point);  // true

var other = new Rect(110.0f, 210.0f, 40.0f, 40.0f);
bool intersects = rect.Intersects(other);  // true
```

### Point Structure

Represents 2D coordinates with vector operations.

```csharp
var p1 = new Point(x: 0.0f, y: 0.0f);
var p2 = new Point(x: 3.0f, y: 4.0f);

// Distance calculations
float distance = p1.Distance(p2);  // 5.0
float magnitude = p2.Magnitude;    // Distance from origin

// Vector operations
var sum = p1 + p2;              // new Point(3, 4)
var diff = p2 - p1;             // new Point(3, 4)
var scaled = p2 * 2.0f;         // new Point(6, 8)
var divided = p2 / 2.0f;        // new Point(1.5, 2.0)
```

### Color Structure

Represents RGBA colors.

```csharp
// Creation
var red = new Color(red: 255, green: 0, blue: 0);
var transparent = new Color(255, 0, 0, alpha: 128);

// From 32-bit ARGB
uint argb = 0xFF0000FF;  // Red with full opacity
var color = Color.FromArgb(argb);

// Predefined colors
var black = Color.Black;
var white = Color.White;
var yellow = Color.Yellow;
var cyan = Color.Cyan;
var magenta = Color.Magenta;

// Properties
byte r = color.Red;
byte g = color.Green;
byte b = color.Blue;
byte a = color.Alpha;
float opacity = color.Opacity;  // 0.0 - 1.0

// Conversion
uint argbValue = color.ToArgb();
string hex = color.ToHex();  // "#FF0000"
```

---

## Exception Handling

### Exception Hierarchy

```csharp
PdfException
├── PdfIoException              // File I/O errors
├── PdfParseException           // PDF format errors
├── PdfEncryptionException      // Password/encryption errors
├── PdfInvalidStateException    // Invalid operation
└── UnsupportedFeatureException // Unsupported PDF features
```

### Handling Specific Errors

```csharp
try
{
    using (var doc = PdfDocument.Open("file.pdf"))
    {
        // Operations...
    }
}
catch (PdfIoException ex)
{
    Console.WriteLine($"File not found: {ex.Message}");
}
catch (PdfParseException ex)
{
    Console.WriteLine($"PDF is corrupted: {ex.Message}");
}
catch (PdfEncryptionException ex)
{
    Console.WriteLine($"Wrong password: {ex.Message}");
}
catch (UnsupportedFeatureException ex)
{
    Console.WriteLine($"Feature not supported: {ex.Message}");
}
catch (PdfException ex)
{
    Console.WriteLine($"PDF error: {ex.Message}");
}
```

---

## Best Practices

### Resource Management

```csharp
// ✅ Good: Using statement ensures disposal
using (var doc = PdfDocument.Open("file.pdf"))
{
    var text = doc.ExtractText(0);
}

// ✅ Good: Multiple documents
using (var source = PdfDocument.Open("source.pdf"))
using (var dest = PdfBuilder.Create().FromMarkdown("..."))
{
    // Process
}

// ❌ Avoid: Missing disposal
var doc = PdfDocument.Open("file.pdf");
var text = doc.ExtractText(0);
// doc never disposed
```

### Error Handling

```csharp
// ✅ Good: Specific exception handling
try
{
    using (var doc = PdfDocument.Open(path))
    {
        // ...
    }
}
catch (PdfIoException ex)
{
    logger.LogError($"Cannot open PDF: {ex.Message}");
    // Handle file error
}
catch (PdfParseException ex)
{
    logger.LogError($"PDF corruption detected: {ex.Message}");
    // Handle corruption
}

// ❌ Avoid: Catching too broadly
try
{
    // ...
}
catch (Exception ex)
{
    // Masks specific PDF errors
}
```

### Performance

```csharp
// ✅ Good: Reuse document instance
using (var doc = PdfDocument.Open("file.pdf"))
{
    for (int i = 0; i < doc.PageCount; i++)
    {
        var page = doc.GetPage(i);
        // Process page
    }
}

// ❌ Avoid: Opening document multiple times
for (int i = 0; i < pageCount; i++)
{
    using (var doc = PdfDocument.Open("file.pdf"))  // Inefficient!
    {
        var page = doc.GetPage(i);
    }
}

// ✅ Good: Use collections efficiently
var allElements = page.FindElements();
var textOnly = allElements.OfType<TextElement>().ToList();

// ❌ Avoid: Redundant calls
foreach (var element in page.FindElements())  // Called each iteration!
{
    // ...
}
```

### Async Operations

```csharp
// ✅ Good: Async for I/O operations
public async Task ProcessPdfAsync(string filePath)
{
    using (var doc = PdfDocument.Open(filePath))
    {
        var text = await doc.ExtractTextAsync(0);
        var markdown = await doc.ToMarkdownAsync(0);
        return (text, markdown);
    }
}

// ✅ Good: Parallel processing
var tasks = Enumerable.Range(0, doc.PageCount)
    .Select(i => doc.ExtractTextAsync(i))
    .ToList();
var results = await Task.WhenAll(tasks);

// ❌ Avoid: Async without awaiting
public async Task BadAsync()
{
    var text = doc.ExtractTextAsync(0);  // Not awaited!
    return text;  // Returns Task<string>, not string
}
```

### LINQ Usage

```csharp
// ✅ Good: Efficient LINQ queries
var largeTextElements = allElements
    .OfType<TextElement>()
    .Where(e => e.FontSize > 12)
    .OrderBy(e => e.BoundingBox.Y)
    .ToList();

// ✅ Good: Lazy evaluation
var query = allElements
    .OfType<TextElement>()
    .Where(e => e.FontSize > 12);
    // Not executed yet

foreach (var element in query)  // Executed here
{
    // ...
}

// ❌ Avoid: Multiple iterations
var elements = page.FindElements();
var count1 = elements.OfType<TextElement>().Count();
var count2 = elements.OfType<ImageElement>().Count();  // Iterates again
var count3 = elements.OfType<PathElement>().Count();   // Iterates again

// ✅ Better:
var grouped = page.FindElements()
    .GroupBy(e => e.Type)
    .ToDictionary(g => g.Key, g => g.Count());
```

---

## Advanced Patterns

### Building Complex Documents

```csharp
// Multi-section document with custom formatting
using (var pdf = PdfBuilder.Create()
    .WithTitle("Project Documentation")
    .WithAuthor("Development Team")
    .WithMargins(72, 72, 72, 72)
    .FromMarkdown(@"
# Project Documentation

## Table of Contents
1. Overview
2. Installation
3. Usage

## Overview
Comprehensive guide to PDF processing...

## Installation
```bash
dotnet add package PdfOxide
```

## Usage
See examples below...
    "))
{
    pdf.Save("documentation.pdf");
}
```

### Batch Processing

```csharp
public class PdfBatchProcessor
{
    public async Task ProcessDirectory(string dirPath)
    {
        var pdfFiles = Directory.GetFiles(dirPath, "*.pdf");

        foreach (var file in pdfFiles)
        {
            try
            {
                await ExtractAndConvert(file);
            }
            catch (PdfException ex)
            {
                Console.WriteLine($"Error processing {file}: {ex.Message}");
            }
        }
    }

    private async Task ExtractAndConvert(string pdfPath)
    {
        using (var doc = PdfDocument.Open(pdfPath))
        {
            for (int i = 0; i < doc.PageCount; i++)
            {
                var markdown = await doc.ToMarkdownAsync(i);
                var outputPath = Path.ChangeExtension(pdfPath, ".md");
                File.WriteAllText(outputPath, markdown);
            }
        }
    }
}
```

### Custom Element Processing

```csharp
public class ElementAnalyzer
{
    public void AnalyzePage(PdfDocument doc, int pageIndex)
    {
        var page = doc.GetPage(pageIndex);
        var elements = page.FindElements();

        var stats = new
        {
            TextCount = elements.OfType<TextElement>().Count(),
            ImageCount = elements.OfType<ImageElement>().Count(),
            TableCount = elements.OfType<TableElement>().Count(),
            AverageFontSize = elements
                .OfType<TextElement>()
                .Average(e => e.FontSize),
            TotalImageData = elements
                .OfType<ImageElement>()
                .Sum(e => e.ImageData.Length)
        };

        Console.WriteLine($"Page {pageIndex + 1} Statistics:");
        Console.WriteLine($"  Text elements: {stats.TextCount}");
        Console.WriteLine($"  Images: {stats.ImageCount}");
        Console.WriteLine($"  Tables: {stats.TableCount}");
        Console.WriteLine($"  Average font size: {stats.AverageFontSize:F1}pt");
        Console.WriteLine($"  Total image data: {stats.TotalImageData:N0} bytes");
    }
}
```

### Search and Replace Workflow

```csharp
public class SearchAndReplace
{
    public void ReplaceInDocument(string inputPath, string outputPath,
        Dictionary<string, string> replacements)
    {
        using (var editor = DocumentEditor.Open(inputPath))
        {
            foreach (var (find, replace) in replacements)
            {
                for (int i = 0; i < editor.PageCount; i++)
                {
                    var page = editor.GetPage(i);
                    var results = page.FindText(find, caseSensitive: false);

                    foreach (var result in results)
                    {
                        page.ReplaceText(result, replace);
                    }
                }
            }

            editor.Save(outputPath);
        }
    }
}

// Usage
var replacements = new Dictionary<string, string>
{
    ["2023"] = "2024",
    ["Version 1.0"] = "Version 2.0",
    ["Draft"] = "Final"
};

var tool = new SearchAndReplace();
tool.ReplaceInDocument("draft.pdf", "final.pdf", replacements);
```

---

## Common Scenarios

### Extract All Text

```csharp
public static string ExtractAllText(string pdfPath)
{
    using (var doc = PdfDocument.Open(pdfPath))
    {
        var sb = new StringBuilder();

        for (int i = 0; i < doc.PageCount; i++)
        {
            var text = doc.ExtractText(i);
            sb.AppendLine($"=== Page {i + 1} ===");
            sb.AppendLine(text);
            sb.AppendLine();
        }

        return sb.ToString();
    }
}
```

### Convert PDF to Markdown

```csharp
public static void ConvertToMarkdown(string pdfPath, string mdPath)
{
    var options = new ConversionOptions
    {
        DetectHeadings = true,
        PreserveLayout = false,
        IncludeImages = true
    };

    using (var doc = PdfDocument.Open(pdfPath))
    {
        var markdown = doc.ToMarkdownAll(options);
        File.WriteAllText(mdPath, markdown);
    }
}
```

### Extract Images

```csharp
public static void ExtractImages(string pdfPath, string outputDir)
{
    Directory.CreateDirectory(outputDir);
    int imageCount = 0;

    using (var doc = PdfDocument.Open(pdfPath))
    {
        for (int i = 0; i < doc.PageCount; i++)
        {
            var page = doc.GetPage(i);
            var images = page.FindElements()
                .OfType<ImageElement>()
                .ToList();

            foreach (var img in images)
            {
                var ext = img.Format switch
                {
                    ImageFormat.Jpeg => ".jpg",
                    ImageFormat.Png => ".png",
                    _ => ".bin"
                };

                var filename = Path.Combine(outputDir, $"image_{imageCount++}{ext}");
                File.WriteAllBytes(filename, img.ImageData);
            }
        }
    }
}
```

---

## Summary

The pdf_oxide C# bindings provide a comprehensive, type-safe API for PDF operations. Key points:

- **IDisposable Pattern**: Always use `using` statements
- **Error Handling**: Catch specific exception types
- **Async Support**: Use async for I/O operations
- **LINQ Integration**: Elements and annotations are fully queryable
- **Type Safety**: Geometry types and enums prevent errors
- **Performance**: Reuse document instances and collections
- **Streams**: Support for files, streams, and bytes

For more information, see:
- API Reference Documentation
- GitHub Examples
- API Unit Tests and Benchmarks
- Architecture Guide (Phases 1-4)

**Happy PDF processing!** 🎉
