# C# Phase 2 API Examples - PDF Creation and Editing

This document provides practical examples for using the Phase 2 C# bindings for pdf_oxide.

## Table of Contents

1. [PDF Creation](#pdf-creation)
2. [Document Editing](#document-editing)
3. [Page Access](#page-access)
4. [Advanced Scenarios](#advanced-scenarios)

---

## PDF Creation

### Creating PDFs from Different Formats

#### From Markdown

```csharp
using System;
using PdfOxide.Core;

// Create a PDF from Markdown
var markdown = @"# Welcome to PDF Oxide

This is a **bold** text and this is *italic*.

## Features

- Text extraction
- PDF creation
- Document editing

```";

using (var pdf = Pdf.FromMarkdown(markdown))
{
    pdf.Save("output.pdf");
    Console.WriteLine($"Created PDF with {pdf.PageCount} pages");
}
```

#### From HTML

```csharp
using System;
using PdfOxide.Core;

var html = @"
<html>
    <head><title>My Document</title></head>
    <body>
        <h1>Heading 1</h1>
        <p>This is a paragraph with <b>bold</b> and <i>italic</i> text.</p>
        <ul>
            <li>Item 1</li>
            <li>Item 2</li>
            <li>Item 3</li>
        </ul>
    </body>
</html>";

using (var pdf = Pdf.FromHtml(html))
{
    pdf.Save("from_html.pdf");
}
```

#### From Plain Text

```csharp
using System;
using PdfOxide.Core;

var text = @"This is a simple text document.
It will be converted to PDF.
Multiple lines are preserved.

This is a new paragraph.";

using (var pdf = Pdf.FromText(text))
{
    pdf.Save("from_text.pdf");
    Console.WriteLine($"Page count: {pdf.PageCount}");
}
```

### Saving PDFs to Different Destinations

#### Save to File

```csharp
using (var pdf = Pdf.FromMarkdown("# Hello World"))
{
    pdf.Save("document.pdf");
}
```

#### Save to Byte Array

```csharp
using (var pdf = Pdf.FromMarkdown("# Hello World"))
{
    byte[] pdfBytes = pdf.SaveToBytes();
    System.IO.File.WriteAllBytes("output.pdf", pdfBytes);
}
```

#### Save to Stream

```csharp
using (var pdf = Pdf.FromMarkdown("# Hello World"))
using (var stream = System.IO.File.Create("output.pdf"))
{
    pdf.SaveToStream(stream);
}
```

#### Async Save

```csharp
using System;
using System.Threading.Tasks;
using PdfOxide.Core;

public async Task SavePdfAsync(string content)
{
    using (var pdf = Pdf.FromMarkdown(content))
    {
        await pdf.SaveAsync("document.pdf");
        Console.WriteLine("PDF saved asynchronously");
    }
}

// Usage
await SavePdfAsync("# Async Example");
```

---

## Document Editing

### Opening and Modifying PDFs

#### Edit Metadata

```csharp
using System;
using PdfOxide.Core;

using (var editor = DocumentEditor.Open("existing.pdf"))
{
    Console.WriteLine($"Pages: {editor.PageCount}");
    
    // Modify metadata
    editor.Title = "Updated Title";
    editor.Author = "John Doe";
    editor.Subject = "PDF Editing Example";
    
    Console.WriteLine($"Modified: {editor.IsModified}");
    
    // Save changes
    editor.Save("edited.pdf");
}
```

#### Check Modification Status

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    if (editor.IsModified)
    {
        Console.WriteLine("Document has unsaved changes");
        editor.Save("document.pdf");
    }
    else
    {
        Console.WriteLine("Document is unchanged");
    }
}
```

#### Read Document Information

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    Console.WriteLine($"Source: {editor.SourcePath}");
    
    var (major, minor) = editor.Version;
    Console.WriteLine($"PDF Version: {major}.{minor}");
    
    Console.WriteLine($"Pages: {editor.PageCount}");
    Console.WriteLine($"Title: {editor.Title ?? "(not set)"}");
    Console.WriteLine($"Author: {editor.Author ?? "(not set)"}");
    Console.WriteLine($"Subject: {editor.Subject ?? "(not set)"}");
}
```

#### Update All Metadata Fields

```csharp
using (var editor = DocumentEditor.Open("input.pdf"))
{
    editor.Title = "New Title";
    editor.Author = "Jane Smith";
    editor.Subject = "Updated Subject";
    
    editor.Save("output_with_metadata.pdf");
}
```

---

## Page Access

### Getting Page Information

#### Page Dimensions

```csharp
using (var editor = DocumentEditor.Open("document.pdf"))
{
    for (int i = 0; i < editor.PageCount; i++)
    {
        // Page access would be extended in Phase 3
        // For now, we can access through DocumentEditor
        Console.WriteLine($"Page {i}: Processing...");
    }
}
```

#### Working with Multiple Pages

```csharp
using System;
using PdfOxide.Core;

using (var editor = DocumentEditor.Open("multi_page.pdf"))
{
    int pageCount = editor.PageCount;
    Console.WriteLine($"Total pages: {pageCount}");
    
    // Create a new PDF from the same content with updated metadata
    editor.Title = $"Updated - {DateTime.Now:yyyy-MM-dd}";
    editor.Author = "Processing System";
    
    editor.Save("processed.pdf");
}
```

---

## Advanced Scenarios

### Batch Processing Multiple PDFs

```csharp
using System;
using System.IO;
using PdfOxide.Core;

public class PdfBatchProcessor
{
    public void ProcessPdfsInDirectory(string directory)
    {
        var files = Directory.GetFiles(directory, "*.pdf");
        
        foreach (var file in files)
        {
            using (var editor = DocumentEditor.Open(file))
            {
                // Add processing metadata
                editor.Author = "Batch Processor";
                editor.Subject = $"Processed on {DateTime.Now:yyyy-MM-dd}";
                
                string outputPath = Path.Combine(directory, 
                    Path.GetFileNameWithoutExtension(file) + "_processed.pdf");
                
                editor.Save(outputPath);
                Console.WriteLine($"Processed: {file}");
            }
        }
    }
}

// Usage
var processor = new PdfBatchProcessor();
processor.ProcessPdfsInDirectory(@"C:\Documents\PDFs");
```

### Convert and Merge Workflows

```csharp
using System;
using System.Collections.Generic;
using PdfOxide.Core;

public class PdfConverter
{
    public void ConvertMarkdownToPdf(string markdownPath, string outputPath)
    {
        string markdown = System.IO.File.ReadAllText(markdownPath);
        
        using (var pdf = Pdf.FromMarkdown(markdown))
        {
            pdf.Save(outputPath);
            Console.WriteLine($"Converted: {markdownPath} -> {outputPath}");
        }
    }
    
    public void ConvertHtmlToPdf(string htmlPath, string outputPath)
    {
        string html = System.IO.File.ReadAllText(htmlPath);
        
        using (var pdf = Pdf.FromHtml(html))
        {
            pdf.Save(outputPath);
            Console.WriteLine($"Converted: {htmlPath} -> {outputPath}");
        }
    }
}

// Usage
var converter = new PdfConverter();
converter.ConvertMarkdownToPdf("document.md", "document.pdf");
converter.ConvertHtmlToPdf("webpage.html", "webpage.pdf");
```

### Error Handling

```csharp
using System;
using PdfOxide.Core;
using PdfOxide.Exceptions;

try
{
    using (var editor = DocumentEditor.Open("document.pdf"))
    {
        editor.Title = "Updated";
        editor.Save("output.pdf");
    }
}
catch (PdfIoException ex)
{
    Console.WriteLine($"File I/O error: {ex.Message}");
}
catch (PdfParseException ex)
{
    Console.WriteLine($"PDF parse error: {ex.Message}");
}
catch (PdfException ex)
{
    Console.WriteLine($"PDF error: {ex.Message}");
}
```

### Async Workflow

```csharp
using System;
using System.Threading.Tasks;
using PdfOxide.Core;

public class AsyncPdfProcessor
{
    public async Task ProcessPdfAsync(string inputPath, string outputPath)
    {
        // Create from Markdown
        var markdown = await System.IO.File.ReadAllTextAsync(inputPath);
        
        using (var pdf = Pdf.FromMarkdown(markdown))
        {
            await pdf.SaveAsync(outputPath);
            Console.WriteLine("Processing completed");
        }
    }
    
    public async Task EditPdfAsync(string inputPath, string outputPath)
    {
        using (var editor = DocumentEditor.Open(inputPath))
        {
            editor.Title = "Async Processed";
            editor.Author = "Async Processor";
            
            await editor.SaveAsync(outputPath);
            Console.WriteLine("Editing completed");
        }
    }
}

// Usage
var processor = new AsyncPdfProcessor();
await processor.ProcessPdfAsync("input.md", "output.pdf");
await processor.EditPdfAsync("document.pdf", "edited.pdf");
```

### Stream-Based Processing

```csharp
using System;
using System.IO;
using PdfOxide.Core;

public class StreamProcessor
{
    public void ConvertStreamToPdf(Stream inputStream, Stream outputStream)
    {
        // Read from input stream
        using (var reader = new StreamReader(inputStream))
        {
            string content = reader.ReadToEnd();
            
            // Create PDF
            using (var pdf = Pdf.FromText(content))
            {
                // Save to output stream
                pdf.SaveToStream(outputStream);
            }
        }
    }
    
    public void EditFromStream(Stream inputStream, Stream outputStream)
    {
        // Read PDF from stream
        using (var editor = DocumentEditor.Open(inputStream))
        {
            editor.Title = "Stream Processed";
            
            // Note: Future enhancement - save to stream support
            string tempFile = Path.GetTempFileName();
            editor.Save(tempFile);
            
            // Copy to output stream
            using (var fileStream = File.OpenRead(tempFile))
            {
                fileStream.CopyTo(outputStream);
            }
            
            File.Delete(tempFile);
        }
    }
}
```

---

## Summary

Phase 2 provides:

✅ **PDF Creation** - From Markdown, HTML, and plain text
✅ **Document Editing** - Metadata modification and state tracking
✅ **Page Access** - Page information and properties
✅ **Async Support** - Asynchronous save operations
✅ **Error Handling** - Typed exception hierarchy
✅ **Stream Support** - Work with files and memory streams

### Key Patterns

- **Creating PDFs**: Use static factory methods (`Pdf.FromMarkdown()`, etc.)
- **Editing PDFs**: Use `DocumentEditor.Open()` and save changes
- **Async Operations**: Use `SaveAsync()` with `CancellationToken`
- **Resource Management**: Always use `using` statements
- **Error Handling**: Catch specific exception types

### Next Steps (Phase 3+)

- DOM element access and manipulation
- Text finding and replacement
- Image handling
- Annotation support
- Advanced page operations (add, remove, reorder)
