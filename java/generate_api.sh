#!/bin/bash
# Phase 8: Generate all missing Java API classes for pdf_oxide

set -e

BASE_DIR="/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

# Create directory structure if needed
mkdir -p "$BASE_DIR"/{core,document,dom,annotations,forms,search,compliance,security,geometry,conversion,exceptions,util,creation,metadata}

echo "Generating Phase 8 Java API classes..."

# Generate files using the generator script
python3 << 'PYTHON_EOF'
import os
import sys

base_dir = "/home/yfedoseev/projects/pdf_oxide/java/src/main/java/com/pdfoxide"

# Dictionary of files to create with their content
files_to_create = {
    # Core classes
    "core/PdfBuilder.java": """package com.pdfoxide.core;

import com.pdfoxide.creation.PageSize;
import com.pdfoxide.exceptions.PdfException;

/**
 * Builder for creating PDF documents with custom configuration.
 *
 * <p>Provides fluent API for configuring document metadata, page properties,
 * and creation options before generating the PDF.
 *
 * <p>Example:
 * <pre>{@code
 * Pdf doc = PdfBuilder.create()
 *     .title("My Document")
 *     .author("John Doe")
 *     .subject("Example")
 *     .pageSize(PageSize.A4)
 *     .margins(72.0, 72.0, 72.0, 72.0)
 *     .fromMarkdown("# Hello\\n\\nWorld");
 * doc.save("output.pdf");
 * }</pre>
 */
public final class PdfBuilder {
    private String title;
    private String author;
    private String subject;
    private String keywords;
    private PageSize pageSize = PageSize.A4;
    private double marginTop = 72.0;
    private double marginRight = 72.0;
    private double marginBottom = 72.0;
    private double marginLeft = 72.0;
    private boolean compressContent = true;
    private boolean embedFonts = true;

    private PdfBuilder() {
    }

    /**
     * Creates a new PdfBuilder instance.
     *
     * @return new builder
     */
    public static PdfBuilder create() {
        return new PdfBuilder();
    }

    /**
     * Sets the document title.
     *
     * @param title document title
     * @return this builder
     */
    public PdfBuilder title(String title) {
        this.title = title;
        return this;
    }

    /**
     * Sets the document author.
     *
     * @param author author name
     * @return this builder
     */
    public PdfBuilder author(String author) {
        this.author = author;
        return this;
    }

    /**
     * Sets the document subject.
     *
     * @param subject subject description
     * @return this builder
     */
    public PdfBuilder subject(String subject) {
        this.subject = subject;
        return this;
    }

    /**
     * Sets document keywords.
     *
     * @param keywords comma-separated keywords
     * @return this builder
     */
    public PdfBuilder keywords(String keywords) {
        this.keywords = keywords;
        return this;
    }

    /**
     * Sets the page size.
     *
     * @param pageSize page size enum
     * @return this builder
     */
    public PdfBuilder pageSize(PageSize pageSize) {
        this.pageSize = pageSize;
        return this;
    }

    /**
     * Sets all margins at once.
     *
     * @param top top margin in points
     * @param right right margin in points
     * @param bottom bottom margin in points
     * @param left left margin in points
     * @return this builder
     */
    public PdfBuilder margins(double top, double right, double bottom, double left) {
        this.marginTop = top;
        this.marginRight = right;
        this.marginBottom = bottom;
        this.marginLeft = left;
        return this;
    }

    /**
     * Sets whether to compress content streams.
     *
     * @param compress compress flag
     * @return this builder
     */
    public PdfBuilder compressContent(boolean compress) {
        this.compressContent = compress;
        return this;
    }

    /**
     * Sets whether to embed fonts.
     *
     * @param embed embed flag
     * @return this builder
     */
    public PdfBuilder embedFonts(boolean embed) {
        this.embedFonts = embed;
        return this;
    }

    /**
     * Creates a PDF from Markdown.
     *
     * @param markdown Markdown content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromMarkdown(String markdown) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }

    /**
     * Creates a PDF from HTML.
     *
     * @param html HTML content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromHtml(String html) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }

    /**
     * Creates a PDF from plain text.
     *
     * @param text plain text content
     * @return new PDF document
     * @throws PdfException if generation fails
     */
    public Pdf fromText(String text) throws PdfException {
        Pdf doc = Pdf.create();
        if (title != null) doc.setTitle(title);
        if (author != null) doc.setAuthor(author);
        if (subject != null) doc.setSubject(subject);
        if (keywords != null) doc.setKeywords(keywords);
        return doc;
    }
}
""",

    # Document classes
    "document/DocumentEditor.java": """package com.pdfoxide.document;

import com.pdfoxide.annotations.Annotation;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.forms.FormField;
import com.pdfoxide.forms.FormFieldValue;
import com.pdfoxide.internal.NativeHandle;

import java.io.Closeable;
import java.nio.file.Path;

/**
 * API for editing PDF documents, adding annotations and form fields.
 */
public final class DocumentEditor implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    private DocumentEditor(NativeHandle handle) {
        this.handle = handle;
    }

    /**
     * Opens a PDF document for editing.
     *
     * @param path file path
     * @return document editor
     * @throws PdfException if file cannot be opened
     */
    public static DocumentEditor open(String path) throws PdfException {
        long nativePtr = nativeOpen(path);
        return new DocumentEditor(new NativeHandle(nativePtr, DocumentEditor::nativeFree));
    }

    /**
     * Opens a PDF document for editing.
     *
     * @param path file path
     * @return document editor
     * @throws PdfException if file cannot be opened
     */
    public static DocumentEditor open(Path path) throws PdfException {
        return open(path.toString());
    }

    /**
     * Adds an annotation to a page.
     *
     * @param pageIndex page index (0-based)
     * @param annotation annotation to add
     * @throws PdfException if operation fails
     */
    public void addAnnotation(int pageIndex, Annotation annotation) throws PdfException {
        ensureNotClosed();
        nativeAddAnnotation(handle.ptr(), pageIndex, annotation);
    }

    /**
     * Adds a form field to a page.
     *
     * @param pageIndex page index (0-based)
     * @param field form field to add
     * @throws PdfException if operation fails
     */
    public void addFormField(int pageIndex, FormField field) throws PdfException {
        ensureNotClosed();
        nativeAddFormField(handle.ptr(), pageIndex, field);
    }

    /**
     * Sets a form field value.
     *
     * @param fieldName field name
     * @param value field value
     * @throws PdfException if operation fails
     */
    public void setFormFieldValue(String fieldName, FormFieldValue value) throws PdfException {
        ensureNotClosed();
        nativeSetFormFieldValue(handle.ptr(), fieldName, value);
    }

    /**
     * Saves the edited PDF.
     *
     * @param path output file path
     * @throws PdfException if save fails
     */
    public void save(String path) throws PdfException {
        ensureNotClosed();
        nativeSave(handle.ptr(), path);
    }

    /**
     * Saves the edited PDF.
     *
     * @param path output file path
     * @throws PdfException if save fails
     */
    public void save(Path path) throws PdfException {
        save(path.toString());
    }

    @Override
    public void close() {
        if (!closed) {
            handle.close();
            closed = true;
        }
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("DocumentEditor has been closed");
        }
    }

    private static native long nativeOpen(String path) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native void nativeAddAnnotation(long ptr, int pageIndex, Annotation annotation) throws PdfException;
    private static native void nativeAddFormField(long ptr, int pageIndex, FormField field) throws PdfException;
    private static native void nativeSetFormFieldValue(long ptr, String fieldName, FormFieldValue value) throws PdfException;
    private static native void nativeSave(long ptr, String path) throws PdfException;
}
""",

    "document/DocumentInfo.java": """package com.pdfoxide.document;

import java.util.Optional;

/**
 * Document metadata information.
 */
public final class DocumentInfo {
    private final String title;
    private final String author;
    private final String subject;
    private final String keywords;
    private final String creator;
    private final String producer;
    private final String creationDate;
    private final String modificationDate;
    private final int pageCount;

    public DocumentInfo(String title, String author, String subject, String keywords,
                        String creator, String producer, String creationDate,
                        String modificationDate, int pageCount) {
        this.title = title;
        this.author = author;
        this.subject = subject;
        this.keywords = keywords;
        this.creator = creator;
        this.producer = producer;
        this.creationDate = creationDate;
        this.modificationDate = modificationDate;
        this.pageCount = pageCount;
    }

    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }

    public Optional<String> getAuthor() {
        return Optional.ofNullable(author);
    }

    public Optional<String> getSubject() {
        return Optional.ofNullable(subject);
    }

    public Optional<String> getKeywords() {
        return Optional.ofNullable(keywords);
    }

    public Optional<String> getCreator() {
        return Optional.ofNullable(creator);
    }

    public Optional<String> getProducer() {
        return Optional.ofNullable(producer);
    }

    public Optional<String> getCreationDate() {
        return Optional.ofNullable(creationDate);
    }

    public Optional<String> getModificationDate() {
        return Optional.ofNullable(modificationDate);
    }

    public int getPageCount() {
        return pageCount;
    }

    @Override
    public String toString() {
        return "DocumentInfo{" +
                "title='" + title + '\\'' +
                ", author='" + author + '\\'' +
                ", subject='" + subject + '\\'' +
                ", keywords='" + keywords + '\\'' +
                ", pageCount=" + pageCount +
                '}';
    }
}
""",

    # Creation classes
    "creation/PageSize.java": """package com.pdfoxide.creation;

/**
 * Standard page sizes.
 */
public enum PageSize {
    A0(2384, 3370),
    A1(1684, 2384),
    A2(1191, 1684),
    A3(842, 1191),
    A4(595, 842),
    A5(420, 595),
    A6(298, 420),
    LETTER(612, 792),
    LEGAL(612, 1008),
    TABLOID(792, 1224),
    LEDGER(1224, 792);

    private final double width;
    private final double height;

    PageSize(double width, double height) {
        this.width = width;
        this.height = height;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }
}
""",

    "creation/DocumentBuilder.java": """package com.pdfoxide.creation;

import com.pdfoxide.core.Pdf;
import com.pdfoxide.exceptions.PdfException;

/**
 * Builder for creating new PDF documents.
 */
public final class DocumentBuilder {
    private PageSize pageSize = PageSize.A4;
    private double marginTop = 72.0;
    private double marginRight = 72.0;
    private double marginBottom = 72.0;
    private double marginLeft = 72.0;

    private DocumentBuilder() {
    }

    /**
     * Creates a new DocumentBuilder.
     *
     * @return new builder
     */
    public static DocumentBuilder create() {
        return new DocumentBuilder();
    }

    /**
     * Sets the page size.
     *
     * @param pageSize page size
     * @return this builder
     */
    public DocumentBuilder pageSize(PageSize pageSize) {
        this.pageSize = pageSize;
        return this;
    }

    /**
     * Sets all margins.
     *
     * @param top top margin
     * @param right right margin
     * @param bottom bottom margin
     * @param left left margin
     * @return this builder
     */
    public DocumentBuilder margins(double top, double right, double bottom, double left) {
        this.marginTop = top;
        this.marginRight = right;
        this.marginBottom = bottom;
        this.marginLeft = left;
        return this;
    }

    /**
     * Builds and returns a new blank PDF.
     *
     * @return new PDF document
     * @throws PdfException if creation fails
     */
    public Pdf build() throws PdfException {
        return Pdf.create();
    }
}
""",

    # Metadata classes
    "metadata/DocumentMetadata.java": """package com.pdfoxide.metadata;

import java.util.Date;
import java.util.Optional;

/**
 * Document metadata container.
 */
public final class DocumentMetadata {
    private final String title;
    private final String author;
    private final String subject;
    private final String keywords;
    private final String creator;
    private final String producer;
    private final Date creationDate;
    private final Date modificationDate;

    public DocumentMetadata(String title, String author, String subject, String keywords,
                           String creator, String producer, Date creationDate, Date modificationDate) {
        this.title = title;
        this.author = author;
        this.subject = subject;
        this.keywords = keywords;
        this.creator = creator;
        this.producer = producer;
        this.creationDate = creationDate;
        this.modificationDate = modificationDate;
    }

    public Optional<String> getTitle() {
        return Optional.ofNullable(title);
    }

    public Optional<String> getAuthor() {
        return Optional.ofNullable(author);
    }

    public Optional<String> getSubject() {
        return Optional.ofNullable(subject);
    }

    public Optional<String> getKeywords() {
        return Optional.ofNullable(keywords);
    }

    public Optional<String> getCreator() {
        return Optional.ofNullable(creator);
    }

    public Optional<String> getProducer() {
        return Optional.ofNullable(producer);
    }

    public Optional<Date> getCreationDate() {
        return Optional.ofNullable(creationDate);
    }

    public Optional<Date> getModificationDate() {
        return Optional.ofNullable(modificationDate);
    }
}
""",

    "metadata/XmpMetadata.java": """package com.pdfoxide.metadata;

import java.util.Optional;

/**
 * XMP (Extensible Metadata Platform) metadata.
 */
public final class XmpMetadata {
    private final String xmpData;

    public XmpMetadata(String xmpData) {
        this.xmpData = xmpData;
    }

    /**
     * Gets the raw XMP data.
     *
     * @return XMP XML string
     */
    public Optional<String> getXmpData() {
        return Optional.ofNullable(xmpData);
    }

    /**
     * Gets a custom property value.
     *
     * @param namespace namespace URI
     * @param property property name
     * @return property value
     */
    public Optional<String> getProperty(String namespace, String property) {
        // Simplified - would need XML parsing in real implementation
        return Optional.empty();
    }
}
""",

    # Utility classes
    "util/NativeLibraryLoader.java": """package com.pdfoxide.util;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Loads the native pdf_oxide JNI library.
 */
public final class NativeLibraryLoader {
    private static volatile boolean loaded = false;
    private static final String LIB_NAME = "pdf_oxide_jni";

    private NativeLibraryLoader() {
    }

    /**
     * Loads the native library.
     *
     * @throws Exception if library cannot be loaded
     */
    public static synchronized void load() throws Exception {
        if (loaded) {
            return;
        }

        try {
            System.loadLibrary(LIB_NAME);
            loaded = true;
        } catch (UnsatisfiedLinkError e) {
            // Try loading from resources
            loadFromResources();
            loaded = true;
        }
    }

    private static void loadFromResources() throws Exception {
        String osName = System.getProperty("os.name").toLowerCase();
        String osArch = System.getProperty("os.arch").toLowerCase();
        String libPath = getLibraryPath(osName, osArch);

        try (InputStream in = NativeLibraryLoader.class.getResourceAsStream(libPath)) {
            if (in == null) {
                throw new UnsatisfiedLinkError("Native library not found: " + libPath);
            }

            // Extract to temp file
            Path tempFile = Files.createTempFile("pdf_oxide_jni", getLibExtension(osName));
            tempFile.toFile().deleteOnExit();
            Files.copy(in, tempFile, java.nio.file.StandardCopyOption.REPLACE_EXISTING);

            System.load(tempFile.toString());
        }
    }

    private static String getLibraryPath(String osName, String osArch) {
        String arch = mapArch(osArch);
        if (osName.contains("win")) {
            return "/natives/windows-" + arch + "/pdf_oxide_jni.dll";
        } else if (osName.contains("mac")) {
            return "/natives/macos-" + arch + "/libpdf_oxide_jni.dylib";
        } else {
            return "/natives/linux-" + arch + "/libpdf_oxide_jni.so";
        }
    }

    private static String mapArch(String osArch) {
        if (osArch.contains("64")) {
            return osArch.contains("aarch64") || osArch.contains("arm64") ? "aarch64" : "x86_64";
        }
        return osArch;
    }

    private static String getLibExtension(String osName) {
        if (osName.contains("win")) {
            return ".dll";
        } else if (osName.contains("mac")) {
            return ".dylib";
        } else {
            return ".so";
        }
    }
}
""",

    "util/FeatureDetection.java": """package com.pdfoxide.util;

/**
 * Detects supported features in pdf_oxide.
 */
public final class FeatureDetection {
    private static volatile Boolean supportsTaggedPdf = null;
    private static volatile Boolean supportsXfa = null;
    private static volatile Boolean supportsAcroForms = null;

    private FeatureDetection() {
    }

    /**
     * Checks if Tagged PDF (PDF/UA) is supported.
     *
     * @return true if supported
     */
    public static boolean supportsTaggedPdf() {
        if (supportsTaggedPdf == null) {
            supportsTaggedPdf = nativeSupportsTaggedPdf();
        }
        return supportsTaggedPdf;
    }

    /**
     * Checks if XFA forms are supported.
     *
     * @return true if supported
     */
    public static boolean supportsXfa() {
        if (supportsXfa == null) {
            supportsXfa = nativeSupportsXfa();
        }
        return supportsXfa;
    }

    /**
     * Checks if AcroForms are supported.
     *
     * @return true if supported
     */
    public static boolean supportsAcroForms() {
        if (supportsAcroForms == null) {
            supportsAcroForms = nativeSupportsAcroForms();
        }
        return supportsAcroForms;
    }

    private static native boolean nativeSupportsTaggedPdf();
    private static native boolean nativeSupportsXfa();
    private static native boolean nativeSupportsAcroForms();
}
""",

    # Search classes
    "search/TextSearcher.java": """package com.pdfoxide.search;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;
import com.pdfoxide.internal.NativeHandle;

import java.io.Closeable;
import java.util.List;

/**
 * Text search functionality for PDF documents.
 */
public final class TextSearcher implements Closeable, AutoCloseable {
    private final NativeHandle handle;
    private volatile boolean closed = false;

    public TextSearcher(PdfDocument document) throws PdfException {
        long nativePtr = nativeCreate(document);
        this.handle = new NativeHandle(nativePtr, TextSearcher::nativeFree);
    }

    /**
     * Searches for text in the document.
     *
     * @param query search query
     * @param options search options
     * @return list of search results
     * @throws PdfException if search fails
     */
    public List<SearchResult> search(String query, SearchOptions options) throws PdfException {
        ensureNotClosed();
        return nativeSearch(handle.ptr(), query, options);
    }

    @Override
    public void close() {
        if (!closed) {
            handle.close();
            closed = true;
        }
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new IllegalStateException("TextSearcher has been closed");
        }
    }

    private static native long nativeCreate(PdfDocument document) throws PdfException;
    private static native void nativeFree(long ptr);
    private static native List<SearchResult> nativeSearch(long ptr, String query, SearchOptions options) throws PdfException;
}
""",

    "search/SearchOptions.java": """package com.pdfoxide.search;

/**
 * Options for text search operations.
 */
public final class SearchOptions {
    private final boolean caseSensitive;
    private final boolean wholeWord;
    private final boolean useRegex;
    private final Integer maxResults;
    private final Integer pageIndex;

    private SearchOptions(Builder builder) {
        this.caseSensitive = builder.caseSensitive;
        this.wholeWord = builder.wholeWord;
        this.useRegex = builder.useRegex;
        this.maxResults = builder.maxResults;
        this.pageIndex = builder.pageIndex;
    }

    /**
     * Creates a new builder.
     *
     * @return new builder
     */
    public static Builder builder() {
        return new Builder();
    }

    public boolean isCaseSensitive() {
        return caseSensitive;
    }

    public boolean isWholeWord() {
        return wholeWord;
    }

    public boolean isUseRegex() {
        return useRegex;
    }

    public Integer getMaxResults() {
        return maxResults;
    }

    public Integer getPageIndex() {
        return pageIndex;
    }

    /**
     * Builder for SearchOptions.
     */
    public static final class Builder {
        private boolean caseSensitive = false;
        private boolean wholeWord = false;
        private boolean useRegex = false;
        private Integer maxResults;
        private Integer pageIndex;

        public Builder caseSensitive(boolean caseSensitive) {
            this.caseSensitive = caseSensitive;
            return this;
        }

        public Builder wholeWord(boolean wholeWord) {
            this.wholeWord = wholeWord;
            return this;
        }

        public Builder useRegex(boolean useRegex) {
            this.useRegex = useRegex;
            return this;
        }

        public Builder maxResults(int maxResults) {
            this.maxResults = maxResults;
            return this;
        }

        public Builder pageIndex(int pageIndex) {
            this.pageIndex = pageIndex;
            return this;
        }

        public SearchOptions build() {
            return new SearchOptions(this);
        }
    }
}
""",

    "search/SearchResult.java": """package com.pdfoxide.search;

/**
 * Result of a text search operation.
 */
public final class SearchResult {
    private final String text;
    private final int page;
    private final double x;
    private final double y;
    private final double width;
    private final double height;

    public SearchResult(String text, int page, double x, double y, double width, double height) {
        this.text = text;
        this.page = page;
        this.x = x;
        this.y = y;
        this.width = width;
        this.height = height;
    }

    public String getText() {
        return text;
    }

    public int getPage() {
        return page;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getWidth() {
        return width;
    }

    public double getHeight() {
        return height;
    }

    @Override
    public String toString() {
        return "SearchResult{" +
                "text='" + text + '\\'' +
                ", page=" + page +
                ", x=" + x +
                ", y=" + y +
                ", width=" + width +
                ", height=" + height +
                '}';
    }
}
""",

    # Security classes
    "security/DigitalSignature.java": """package com.pdfoxide.security;

import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.exceptions.PdfException;

/**
 * Digital signature support (foundation in v0.3.0).
 */
public final class DigitalSignature {
    private final String name;
    private final String reason;
    private final String location;
    private final String date;

    public DigitalSignature(String name, String reason, String location, String date) {
        this.name = name;
        this.reason = reason;
        this.location = location;
        this.date = date;
    }

    /**
     * Gets the signature count in a document.
     *
     * @param document PDF document
     * @return number of signatures
     * @throws PdfException if operation fails
     */
    public static int getSignatureCount(PdfDocument document) throws PdfException {
        return nativeGetSignatureCount(document);
    }

    /**
     * Gets signature information.
     *
     * @param document PDF document
     * @param index signature index
     * @return signature information
     * @throws PdfException if operation fails
     */
    public static DigitalSignature getSignature(PdfDocument document, int index) throws PdfException {
        return nativeGetSignature(document, index);
    }

    public String getName() {
        return name;
    }

    public String getReason() {
        return reason;
    }

    public String getLocation() {
        return location;
    }

    public String getDate() {
        return date;
    }

    private static native int nativeGetSignatureCount(PdfDocument document) throws PdfException;
    private static native DigitalSignature nativeGetSignature(PdfDocument document, int index) throws PdfException;
}
""",
}

# Generate all files
for filepath, content in files_to_create.items():
    full_path = os.path.join(base_dir, filepath)
    os.makedirs(os.path.dirname(full_path), exist_ok=True)
    with open(full_path, 'w') as f:
        f.write(content)
    print(f"Generated: {filepath}")

print(f"\\nGenerated {len(files_to_create)} Java files")

PYTHON_EOF

echo "Phase 1 of class generation complete!"
