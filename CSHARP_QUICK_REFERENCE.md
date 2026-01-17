# PDF Oxide C# Bindings - Quick Reference

Fast reference for common PDF operations.

---

## Installation

```bash
dotnet add package PdfOxide
```

---

## Reading PDFs

```csharp
using PdfOxide.Core;

// Open and extract
using (var doc = PdfDocument.Open("file.pdf"))
{
    string text = doc.ExtractText(0);  // Page 0
    var markdown = doc.ToMarkdown(0);
    var html = doc.ToHtml(0);
}

// With password
using (var doc = PdfDocument.OpenWithPassword("secure.pdf", "password"))
{
    // ...
}
```

---

## Creating PDFs

```csharp
using PdfOxide.Core;

// From Markdown
using (var pdf = Pdf.FromMarkdown("# Title\n\nContent"))
{
    pdf.Save("output.pdf");
}

// From HTML
using (var pdf = Pdf.FromHtml("<h1>Title</h1><p>Content</p>"))
{
    pdf.Save("output.pdf");
}

// With builder
using (var pdf = PdfBuilder.Create()
    .WithTitle("Report")
    .WithPageSize(PageSize.A4)
    .WithMargins(72, 72, 72, 72)
    .FromMarkdown("Content"))
{
    pdf.Save("output.pdf");
}
```

---

## Editing PDFs

```csharp
using PdfOxide.Core;

using (var editor = DocumentEditor.Open("input.pdf"))
{
    var page = editor.GetPage(0);

    // Find and replace
    var results = page.FindText("old");
    foreach (var result in results)
    {
        page.ReplaceText(result, "new");
    }

    editor.Save("output.pdf");
}
```

---

## Elements

```csharp
var elements = page.FindElements();

// Filter by type
var textElements = elements.OfType<TextElement>().ToList();
var images = elements.OfType<ImageElement>().ToList();
var tables = elements.OfType<TableElement>().ToList();

// Text element
var text = textElements.First();
string content = text.Content;
float fontSize = text.FontSize;

// Image element
var img = images.First();
var format = img.Format;  // Jpeg, Png, etc.
var (width, height) = img.Dimensions;
byte[] data = img.ImageData;

// Table element
var table = tables.First();
int rows = table.RowCount;
int cols = table.ColumnCount;
string cell = table.GetCellContent(0, 0);
```

---

## Annotations

```csharp
var annotations = page.GetAnnotations();

// Filter by type
var notes = annotations.OfType<TextAnnotation>().ToList();
var links = annotations.OfType<LinkAnnotation>().ToList();
var highlights = annotations.OfType<TextMarkupAnnotation>().ToList();

// Common properties
var annotation = annotations.First();
string contents = annotation.Contents;
string author = annotation.Author;
var bbox = annotation.BoundingBox;
var color = annotation.Color;

// Text annotation
var note = notes.First();
var icon = note.Icon;  // Comment, Key, Note, etc.
bool isOpen = note.IsOpen;

// Link annotation
var link = links.First();
if (link.IsUriLink)
    string uri = link.Uri;
else if (link.IsPageLink)
    int page = link.DestinationPage;

// Text markup
var markup = highlights.First();
bool isHighlight = markup.IsHighlight;
bool isUnderline = markup.IsUnderline;
```

---

## Search

```csharp
// Search on page
var results = page.FindText("term");

foreach (var result in results)
{
    string text = result.Text;
    var bbox = result.BoundingBox;
    int pageIndex = result.PageIndex;
}

// Case-insensitive
var results = page.FindText("term", caseSensitive: false);

// Document-wide
var allResults = new List<SearchResult>();
for (int i = 0; i < doc.PageCount; i++)
{
    var pageResults = doc.GetPage(i).FindText("term");
    allResults.AddRange(pageResults);
}
```

---

## Geometry

```csharp
using PdfOxide.Geometry;

// Rectangle
var rect = new Rect(100, 200, 50, 30);
float right = rect.Right;      // X + Width
float bottom = rect.Bottom;    // Y + Height
bool contains = rect.Contains(point);

// Point
var p1 = new Point(0, 0);
var p2 = new Point(3, 4);
float distance = p1.Distance(p2);  // 5.0
var sum = p1 + p2;

// Color
var red = new Color(255, 0, 0);
var rgb = Color.FromArgb(0xFF0000FF);
string hex = red.ToHex();  // "#FF0000"
```

---

## Exceptions

```csharp
try
{
    using (var doc = PdfDocument.Open("file.pdf"))
    {
        // ...
    }
}
catch (PdfIoException ex)
{
    // File not found, permission denied, etc.
}
catch (PdfParseException ex)
{
    // Invalid PDF structure
}
catch (PdfEncryptionException ex)
{
    // Wrong password
}
catch (PdfException ex)
{
    // Generic PDF error
}
```

---

## Properties

| Class | Property | Returns | Notes |
|-------|----------|---------|-------|
| PdfDocument | Version | (byte, byte) | Major, minor version |
| | PageCount | int | Total pages |
| | HasStructureTree | bool | Tagged PDF |
| TextElement | Content | string | Text content |
| | FontSize | float | Font size in points |
| ImageElement | Format | ImageFormat | Jpeg, Png, etc. |
| | Dimensions | (int, int) | Width, height in pixels |
| | ImageData | byte[] | Raw image bytes |
| TableElement | RowCount | int | Number of rows |
| | ColumnCount | int | Number of columns |
| Annotation | Contents | string | Annotation text |
| | Author | string | Creator |
| | BoundingBox | Rect | Position and size |
| | Color | Color? | RGBA color |
| SearchResult | Text | string | Matched text |
| | BoundingBox | Rect | Match location |
| | PageIndex | int | Page number (0-based) |

---

## LINQ Examples

```csharp
// Filter elements
var largeText = elements
    .OfType<TextElement>()
    .Where(e => e.FontSize > 14)
    .ToList();

// Group annotations
var byColor = annotations
    .Where(a => a.Color != null)
    .GroupBy(a => a.Color.Value)
    .ToDictionary(g => g.Key, g => g.Count());

// Find and sort search results
var sorted = results
    .OrderBy(r => r.PageIndex)
    .ThenBy(r => r.BoundingBox.Y)
    .ToList();

// Count specific types
int imageCount = elements.OfType<ImageElement>().Count();
```

---

## Async Patterns

```csharp
// Async extraction
var text = await doc.ExtractTextAsync(0);

// Async save
await pdf.SaveAsync("output.pdf");

// With cancellation
var cts = new CancellationTokenSource(TimeSpan.FromSeconds(30));
var text = await doc.ExtractTextAsync(0, cts.Token);

// Parallel processing
var tasks = Enumerable.Range(0, doc.PageCount)
    .Select(i => doc.ExtractTextAsync(i))
    .ToList();
var results = await Task.WhenAll(tasks);
```

---

## Common Patterns

### Batch Processing
```csharp
var files = Directory.GetFiles(".", "*.pdf");
foreach (var file in files)
{
    using (var doc = PdfDocument.Open(file))
    {
        var md = doc.ToMarkdownAll();
        File.WriteAllText(Path.ChangeExtension(file, ".md"), md);
    }
}
```

### Extract All Images
```csharp
using (var doc = PdfDocument.Open("file.pdf"))
{
    for (int i = 0; i < doc.PageCount; i++)
    {
        var images = doc.GetPage(i).FindElements()
            .OfType<ImageElement>()
            .ToList();

        foreach (var img in images)
        {
            var ext = img.Format == ImageFormat.Jpeg ? ".jpg" : ".png";
            File.WriteAllBytes($"image_{i}{ext}", img.ImageData);
        }
    }
}
```

### Search and List
```csharp
using (var doc = PdfDocument.Open("file.pdf"))
{
    for (int i = 0; i < doc.PageCount; i++)
    {
        var results = doc.GetPage(i).FindText("keyword");
        if (results.Any())
        {
            Console.WriteLine($"Page {i + 1}: {results.Count()} matches");
        }
    }
}
```

---

## Tips

- Always use `using` statements for automatic cleanup
- Reuse document instances across multiple operations
- Use LINQ for efficient element/annotation filtering
- Catch specific exception types for better error handling
- Use async operations for I/O-bound tasks
- Collections are LINQ-queryable (ToList() if needed)
- Check element/annotation `Type` before casting
- Images may be empty - check `ImageData.Length > 0`
- Search is case-sensitive by default

---

## Links

- **Full API Guide**: CSHARP_API_GUIDE.md
- **Test Examples**: csharp/PdfOxide.Tests/*.cs
- **Benchmarks**: csharp/PdfOxide.Benchmarks/*.cs
- **Architecture**: CSHARP_PHASE*.md files
