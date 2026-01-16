# Complete List of Generated Java API Classes

**Generated**: January 16, 2026
**Total**: 135 Classes across 15 packages

## com.pdfoxide.annotations (53 classes)

### Annotation Base Classes
- `Annotation.java` - Base class for all annotations
- `AnnotationAction.java` - Base class for annotation actions
- `AnnotationFlags.java` - Annotation display flags

### Annotation Implementations
- `TextAnnotation.java` - Text/comment annotations
- `HighlightAnnotation.java` - Highlight annotations
- `UnderlineAnnotation.java` - Underline annotations
- `StrikeOutAnnotation.java` - Strike-out annotations
- `SquigglyAnnotation.java` - Squiggly underline annotations
- `LinkAnnotation.java` - Hyperlink annotations
- `StampAnnotation.java` - Stamp annotations
- `WatermarkAnnotation.java` - Watermark annotations
- `LineAnnotation.java` - Line annotations
- `SquareAnnotation.java` - Square/rectangle annotations
- `CircleAnnotation.java` - Circle/oval annotations
- `PolygonAnnotation.java` - Polygon annotations
- `InkAnnotation.java` - Freehand ink annotations
- `RedactAnnotation.java` - Redaction annotations
- `CaretAnnotation.java` - Caret insertion annotations
- `FreeTextAnnotation.java` - Free text annotations
- `FileAttachmentAnnotation.java` - File attachment annotations
- `SoundAnnotation.java` - Sound/audio annotations
- `MovieAnnotation.java` - Movie/video annotations
- `ScreenAnnotation.java` - Screen/media annotations
- `RichMediaAnnotation.java` - Rich media annotations
- `ThreeDAnnotation.java` - 3D model annotations
- `PopupAnnotation.java` - Popup annotations

### Annotation Builders
- `TextAnnotationBuilder.java`
- `HighlightAnnotationBuilder.java`
- `LinkAnnotationBuilder.java`
- `StampAnnotationBuilder.java`
- `WatermarkAnnotationBuilder.java`
- `LineAnnotationBuilder.java`
- `SquareAnnotationBuilder.java`
- `CircleAnnotationBuilder.java`
- `PolygonAnnotationBuilder.java`
- `InkAnnotationBuilder.java`
- `RedactAnnotationBuilder.java`
- `CaretAnnotationBuilder.java`
- `FreeTextAnnotationBuilder.java`
- `FileAttachmentAnnotationBuilder.java`
- `SoundAnnotationBuilder.java`
- `MovieAnnotationBuilder.java`
- `ScreenAnnotationBuilder.java`
- `RichMediaAnnotationBuilder.java`
- `ThreeDAnnotationBuilder.java`
- `PopupAnnotationBuilder.java`

### Annotation Supporting Classes
- `LinkAction.java` - Link action types
- `LaunchAction.java` - Launch action
- `GoToAction.java` - GoTo action
- `StampType.java` - Stamp type enumeration
- `HighlightMode.java` - Highlight mode enumeration
- `Caret.java` - Caret type enumeration
- `FileAttachmentIcon.java` - File attachment icon enumeration

## com.pdfoxide.compliance (9 classes)

- `PdfAValidator.java` - PDF/A compliance validator
- `ValidationResult.java` - Validation results container
- `ValidationStats.java` - Validation statistics
- `ComplianceError.java` - Compliance error representation
- `ComplianceWarning.java` - Compliance warning representation
- `PdfALevel.java` - PDF/A level enumeration
- `PdfAPart.java` - PDF/A part variant enumeration
- `ErrorCode.java` - Error code enumeration
- `WarningCode.java` - Warning code enumeration

## com.pdfoxide.conversion (4 classes)

- `ConversionOptions.java` - Main conversion options
- `ConversionOptionsBuilder.java` - Builder for conversion options
- `MarkdownOptions.java` - Markdown-specific options
- `HtmlOptions.java` - HTML-specific options

## com.pdfoxide.core (3 classes)

- `Pdf.java` - Main unified PDF API (EXISTING)
- `PdfDocument.java` - Read-only PDF interface (EXISTING)
- `PdfBuilder.java` - Fluent builder for PDF creation

## com.pdfoxide.creation (2 classes)

- `DocumentBuilder.java` - Builder for creating new documents
- `PageSize.java` - Standard page size enumeration

## com.pdfoxide.document (2 classes)

- `DocumentEditor.java` - PDF editing interface
- `DocumentInfo.java` - Document metadata container

## com.pdfoxide.dom (14 classes)

### Core DOM Classes
- `PdfPage.java` - Page representation and navigation
- `PdfElement.java` - Base class for page elements
- `PdfText.java` - Text element
- `PdfImage.java` - Image element
- `PdfPath.java` - Vector path element
- `PdfTable.java` - Table element
- `PdfStructure.java` - Logical structure (Tagged PDF)

### Supporting DOM Classes
- `ElementId.java` - Element identifier
- `TextContent.java` - Text content model
- `ImageContent.java` - Image content model
- `TextStyle.java` - Text styling information
- `PageMetrics.java` - Page size and rotation
- `TableCell.java` - Table cell representation
- `ContentType.java` - Content type enumeration

## com.pdfoxide.exceptions (7 classes)

- `PdfException.java` - Base exception class
- `ParseException.java` - PDF parsing error
- `EncryptionException.java` - Encryption-related error
- `IoException.java` - File I/O error
- `InvalidStateException.java` - Invalid state error
- `UnsupportedFeatureException.java` - Unsupported feature error
- `ExceptionUtils.java` - Exception utility methods

## com.pdfoxide.forms (21 classes)

### Form Field Types
- `TextField.java` - Text input field
- `CheckboxField.java` - Checkbox field
- `ComboBoxField.java` - Dropdown combo box
- `ListBoxField.java` - List box field
- `RadioButtonField.java` - Radio button group
- `PushButtonField.java` - Push button
- `SignatureField.java` - Digital signature field

### Form Field Builders
- `TextFieldBuilder.java`
- `CheckboxBuilder.java`
- `ComboBoxBuilder.java`
- `ListBoxBuilder.java`
- `RadioButtonGroupBuilder.java`
- `RadioButtonBuilder.java` - Helper for radio button options
- `PushButtonBuilder.java`
- `SignatureFieldBuilder.java`

### Form Supporting Classes
- `FormField.java` - Base form field interface
- `FormFieldValue.java` - Form field value container
- `FormFieldType.java` - Form field type enumeration
- `BorderStyle.java` - Form field border styling
- `ButtonAction.java` - Push button action types
- `FormExtractor.java` - Extract fields from PDF

## com.pdfoxide.geometry (7 classes)

- `Point.java` - 2D point (x, y)
- `Rect.java` - Rectangle (x, y, width, height)
- `Color.java` - RGB color representation
- `Transform.java` - Affine transformation matrix
- `Matrix.java` - 2D transformation matrix (3x3)
- `Dimensions.java` - Width and height pair
- `Margin.java` - Margin values (top, right, bottom, left)

## com.pdfoxide.internal (1 class)

- `NativeHandle.java` - Wrapper for native C pointers (EXISTING)

## com.pdfoxide.metadata (2 classes)

- `DocumentMetadata.java` - Document metadata container
- `XmpMetadata.java` - XMP metadata support

## com.pdfoxide.search (3 classes)

- `TextSearcher.java` - Text search engine
- `SearchOptions.java` - Search configuration options
- `SearchResult.java` - Search result representation

## com.pdfoxide.security (4 classes)

- `DigitalSignature.java` - Digital signature handling
- `SignatureConfig.java` - Signature configuration
- `SignatureConfigBuilder.java` - Builder for signature setup
- `CertificateInfo.java` - Certificate information (EXISTING)

## com.pdfoxide.util (3 classes)

- `NativeLibraryLoader.java` - JNI library loading
- `FeatureDetection.java` - Runtime feature detection
- `PdfVersion.java` - PDF version representation

---

## Generation Summary

| Package | Classes | Builders | Enums | Existing | Total |
|---------|---------|----------|-------|----------|-------|
| annotations | 24 | 20 | 7 | 2 | 53 |
| compliance | 6 | 0 | 3 | 0 | 9 |
| conversion | 2 | 1 | 0 | 1 | 4 |
| core | 1 | 1 | 0 | 2 | 3 |
| creation | 1 | 1 | 1 | 0 | 2 |
| document | 2 | 0 | 0 | 0 | 2 |
| dom | 7 | 0 | 1 | 6 | 14 |
| exceptions | 6 | 1 | 0 | 1 | 7 |
| forms | 8 | 8 | 1 | 4 | 21 |
| geometry | 5 | 0 | 0 | 3 | 7 |
| internal | 0 | 0 | 0 | 1 | 1 |
| metadata | 2 | 0 | 0 | 0 | 2 |
| search | 3 | 0 | 0 | 0 | 3 |
| security | 2 | 1 | 0 | 1 | 4 |
| util | 2 | 0 | 0 | 1 | 3 |
| **TOTAL** | **71** | **33** | **13** | **22** | **135** |

---

## Key Features

### Design Patterns Used
- **Builder Pattern**: 30+ builder classes for complex object creation
- **Factory Pattern**: Static factory methods in Pdf, PdfBuilder, etc.
- **Strategy Pattern**: ConversionOptions, SearchOptions with different configurations
- **Flyweight Pattern**: TextStyle, Color for reusable styling objects
- **Resource Management**: AutoCloseable for proper cleanup

### Type Safety
- Strong typing with generics
- Enum types for fixed option sets
- Proper Optional usage for nullable values
- No raw types or unchecked operations

### Documentation
- Comprehensive Javadoc for all public APIs
- Usage examples in class-level documentation
- Parameter descriptions
- Return value descriptions
- Exception documentation

### Consistency
- Uniform naming conventions
- Consistent method signatures
- Builder patterns applied consistently
- Error handling aligned across packages

---

## File Locations

All generated files are located in:
```
/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide/
```

Organized by package into subdirectories:
- `annotations/` - Annotation framework
- `compliance/` - PDF/A validation
- `conversion/` - Format conversion
- `core/` - Main API
- `creation/` - Document creation
- `document/` - Document editing
- `dom/` - DOM navigation
- `exceptions/` - Exception hierarchy
- `forms/` - Form fields
- `geometry/` - Geometry helpers
- `internal/` - Internal utilities
- `metadata/` - Metadata handling
- `search/` - Text search
- `security/` - Digital signatures
- `util/` - Utility functions

---

## Next Steps

1. Implement native JNI method stubs in Rust
2. Write comprehensive unit tests
3. Create integration tests
4. Build cross-platform native libraries
5. Package JAR with embedded natives
6. Create example programs
7. Write user documentation
8. Performance benchmarking
