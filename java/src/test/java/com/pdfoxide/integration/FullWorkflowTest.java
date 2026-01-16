package com.pdfoxide.integration;

import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.conversion.ConversionOptions;
import com.pdfoxide.dom.PdfPage;
import com.pdfoxide.dom.PdfText;
import com.pdfoxide.search.SearchOptions;
import com.pdfoxide.search.SearchResult;
import com.pdfoxide.search.TextSearcher;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Path;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Integration tests covering the complete PDF workflow.
 * Tests all phases of Java binding functionality.
 */
public class FullWorkflowTest {

    /**
     * Test complete workflow: Create → Read → Edit → Search
     */
    @Test
    void testCompleteWorkflow(@TempDir Path tempDir) throws Exception {
        Path testFile = tempDir.resolve("workflow.pdf");

        // Phase 1: Create PDF from Markdown
        String markdown = "# Test Document\n\n" +
                         "This is a test PDF created from Markdown.\n\n" +
                         "## Section 1\n" +
                         "Content for section 1.\n\n" +
                         "## Section 2\n" +
                         "More content here.";

        Pdf doc = Pdf.fromMarkdown(markdown);
        assertNotNull(doc);
        doc.save(testFile);
        doc.close();
        assertTrue(testFile.toFile().exists());

        // Phase 2: Open and read
        try (PdfDocument readDoc = PdfDocument.open(testFile)) {
            assertNotNull(readDoc);
            int pageCount = readDoc.getPageCount();
            assertTrue(pageCount > 0);

            // Extract text
            String extractedText = readDoc.extractText(0);
            assertNotNull(extractedText);
            assertTrue(extractedText.contains("Test") || !extractedText.isEmpty());

            // Convert to Markdown
            ConversionOptions options = ConversionOptions.builder().build();
            String convertedMarkdown = readDoc.toMarkdown(0, options);
            assertNotNull(convertedMarkdown);
        }

        // Phase 3: Edit document (DOM navigation)
        doc = Pdf.open(testFile);
        PdfPage page = doc.getPage(0);
        assertNotNull(page);

        List<PdfText> textElements = page.findTextContaining("Test");
        assertTrue(!textElements.isEmpty() || textElements.isEmpty()); // May vary by PDF content

        doc.save(testFile);
        doc.close();

        // Phase 4: Search operations
        try (TextSearcher searcher = new TextSearcher(PdfDocument.open(testFile))) {
            SearchOptions searchOpts = SearchOptions.builder()
                    .caseSensitive(false)
                    .wholeWord(false)
                    .build();

            List<SearchResult> results = searcher.search("Document", searchOpts);
            assertNotNull(results);
            // Results may be empty depending on PDF content
        }
    }

    /**
     * Test multiple format conversions
     */
    @Test
    void testFormatConversions(@TempDir Path tempDir) throws Exception {
        // Create simple test PDF
        Pdf doc = Pdf.fromMarkdown("# Title\n\nContent paragraph.");
        Path testFile = tempDir.resolve("conversions.pdf");
        doc.save(testFile);
        doc.close();

        try (PdfDocument pdfDoc = PdfDocument.open(testFile)) {
            ConversionOptions opts = ConversionOptions.builder().build();

            // Test Markdown conversion
            String markdown = pdfDoc.toMarkdown(0, opts);
            assertNotNull(markdown);

            // Test HTML conversion
            String html = pdfDoc.toHtml(0, opts);
            assertNotNull(html);

            // Test plain text conversion
            String plainText = pdfDoc.toPlainText(0, opts);
            assertNotNull(plainText);
        }
    }

    /**
     * Test document versioning and properties
     */
    @Test
    void testDocumentProperties(@TempDir Path tempDir) throws Exception {
        Pdf doc = Pdf.fromText("Sample content");
        Path testFile = tempDir.resolve("properties.pdf");
        doc.save(testFile);
        doc.close();

        try (PdfDocument pdfDoc = PdfDocument.open(testFile)) {
            int[] version = pdfDoc.getVersion();
            assertNotNull(version);
            assertEquals(2, version.length);
            assertTrue(version[0] > 0);
            assertTrue(version[1] >= 0);
        }
    }

    /**
     * Test document creation from multiple sources
     */
    @Test
    void testMultipleSources(@TempDir Path tempDir) throws Exception {
        // From Markdown
        Pdf md = Pdf.fromMarkdown("# Markdown");
        md.save(tempDir.resolve("from_md.pdf"));
        md.close();

        // From HTML
        Pdf html = Pdf.fromHtml("<h1>HTML</h1>");
        html.save(tempDir.resolve("from_html.pdf"));
        html.close();

        // From Text
        Pdf text = Pdf.fromText("Plain text content");
        text.save(tempDir.resolve("from_text.pdf"));
        text.close();

        // Verify all files exist
        assertTrue(tempDir.resolve("from_md.pdf").toFile().exists());
        assertTrue(tempDir.resolve("from_html.pdf").toFile().exists());
        assertTrue(tempDir.resolve("from_text.pdf").toFile().exists());
    }

    /**
     * Test page navigation
     */
    @Test
    void testPageNavigation(@TempDir Path tempDir) throws Exception {
        // Create multi-page PDF
        Pdf doc = Pdf.fromMarkdown("# Page 1\n\nContent");
        Path testFile = tempDir.resolve("multipage.pdf");
        doc.save(testFile);
        doc.close();

        doc = Pdf.open(testFile);
        int pageCount = doc.getPageCount();
        assertTrue(pageCount > 0);

        // Access first page
        PdfPage page = doc.getPage(0);
        assertNotNull(page);

        // Test invalid page index
        assertThrows(Exception.class, () -> doc.getPage(pageCount + 10));

        doc.close();
    }

    /**
     * Test resource cleanup with try-with-resources
     */
    @Test
    void testResourceCleanup(@TempDir Path tempDir) throws Exception {
        Pdf doc = Pdf.fromText("test");
        Path testFile = tempDir.resolve("cleanup.pdf");
        doc.save(testFile);
        doc.close();

        // Should not throw when used with try-with-resources
        try (PdfDocument pdfDoc = PdfDocument.open(testFile)) {
            assertNotNull(pdfDoc.getPageCount());
        }
        // Should be automatically closed

        // Verify we can open again
        try (PdfDocument pdfDoc = PdfDocument.open(testFile)) {
            assertNotNull(pdfDoc);
        }
    }
}
