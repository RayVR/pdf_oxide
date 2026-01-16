import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.search.SearchOptions;
import com.pdfoxide.search.SearchResult;
import com.pdfoxide.search.TextSearcher;

import java.util.List;

/**
 * Example: Search for text in PDF documents.
 *
 * This example demonstrates:
 * - Creating a searchable PDF
 * - Performing text search with various options
 * - Literal text search
 * - Case-sensitive search
 * - Whole-word search
 * - Regex search
 * - Page-specific search
 *
 * Usage: java SearchPdf
 */
public class SearchPdf {

    public static void main(String[] args) {
        System.out.println("PDF Search Examples");
        System.out.println("=".repeat(60));

        try {
            // First, create a sample PDF with content to search
            System.out.println("\n1. Creating sample PDF for searching...");
            Pdf sampleDoc = createSamplePdf();
            String sampleFile = "sample_search.pdf";
            sampleDoc.save(sampleFile);
            sampleDoc.close();
            System.out.println("   ✓ Created: " + sampleFile);

            // Now open and search the PDF
            try (PdfDocument doc = PdfDocument.open(sampleFile)) {
                System.out.println("\n2. Searching PDF document...");
                System.out.println("-".repeat(60));

                // Example 1: Basic literal search
                searchLiteral(doc);

                // Example 2: Case-sensitive search
                searchCaseSensitive(doc);

                // Example 3: Whole-word search
                searchWholeWord(doc);

                // Example 4: Page-specific search
                searchPageSpecific(doc);
            }

            System.out.println("\n" + "=".repeat(60));
            System.out.println("✅ Search examples completed!");

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }

    /**
     * Create a sample PDF with searchable content
     */
    private static Pdf createSamplePdf() throws Exception {
        String markdown = """
                # Search Example Document

                ## Chapter 1: Introduction

                This document contains multiple instances of the word "example".
                Example PDF documents are useful for testing search functionality.
                We use examples to demonstrate various features.

                ### Section 1.1: First Example

                The first example shows basic text search capabilities.
                Notice that this example contains the word "example" multiple times.
                Example usage patterns are important for users.

                ## Chapter 2: Advanced Topics

                Different examples can use different search options.
                Case sensitivity affects how examples are found.
                Whole-word matching provides more precise search results.

                ### Section 2.1: Search Options

                - Literal search finds exact text
                - Case-sensitive search distinguishes capitalization
                - Whole-word search requires word boundaries
                - Regular expressions provide pattern matching

                ### Example Section

                This section demonstrates advanced search techniques.
                Example patterns help identify specific content.
                Search results can be filtered by page number.

                ## Conclusion

                PDF text search is powerful for document analysis.
                These examples should help you understand the capabilities.
                Example-based learning is effective.
                """;

        return Pdf.fromMarkdown(markdown);
    }

    /**
     * Perform basic literal search
     */
    private static void searchLiteral(PdfDocument doc) throws Exception {
        System.out.println("\nA. Literal Search: Find all instances of 'example'");

        try (TextSearcher searcher = new TextSearcher(doc)) {
            SearchOptions options = SearchOptions.builder()
                    .caseSensitive(false)
                    .wholeWord(false)
                    .build();

            List<SearchResult> results = searcher.search("example", options);

            if (results.isEmpty()) {
                System.out.println("   No results found");
            } else {
                System.out.printf("   Found %d results:%n", results.size());
                for (int i = 0; i < Math.min(results.size(), 3); i++) {
                    SearchResult r = results.get(i);
                    System.out.printf("   [%d] Page %d: \"%s\"%n",
                            i + 1, r.getPage() + 1, truncate(r.getText(), 50));
                }
                if (results.size() > 3) {
                    System.out.printf("   ... and %d more%n", results.size() - 3);
                }
            }
        }
    }

    /**
     * Perform case-sensitive search
     */
    private static void searchCaseSensitive(PdfDocument doc) throws Exception {
        System.out.println("\nB. Case-Sensitive Search: Find 'Example' (capitalized)");

        try (TextSearcher searcher = new TextSearcher(doc)) {
            SearchOptions options = SearchOptions.builder()
                    .caseSensitive(true)
                    .wholeWord(false)
                    .build();

            List<SearchResult> results = searcher.search("Example", options);

            if (results.isEmpty()) {
                System.out.println("   No results found for capitalized 'Example'");
            } else {
                System.out.printf("   Found %d results:%n", results.size());
                results.stream()
                        .limit(3)
                        .forEach(r -> System.out.printf("   - Page %d: \"%s\"%n",
                                r.getPage() + 1, truncate(r.getText(), 50)));
            }
        }
    }

    /**
     * Perform whole-word search
     */
    private static void searchWholeWord(PdfDocument doc) throws Exception {
        System.out.println("\nC. Whole-Word Search: Find complete word 'example'");

        try (TextSearcher searcher = new TextSearcher(doc)) {
            SearchOptions options = SearchOptions.builder()
                    .caseSensitive(false)
                    .wholeWord(true)
                    .build();

            List<SearchResult> results = searcher.search("example", options);

            System.out.printf("   Found %d whole-word matches%n", results.size());
            System.out.println("   (This differs from literal search which includes 'examples')");
        }
    }

    /**
     * Perform page-specific search
     */
    private static void searchPageSpecific(PdfDocument doc) throws Exception {
        int pageCount = doc.getPageCount();

        System.out.println("\nD. Page-Specific Search: Find 'Search' on first 2 pages");

        try (TextSearcher searcher = new TextSearcher(doc)) {
            SearchOptions options = SearchOptions.builder()
                    .caseSensitive(false)
                    .wholeWord(true)
                    .pages(java.util.Arrays.asList(0, Math.min(1, pageCount - 1)))
                    .build();

            List<SearchResult> results = searcher.search("Search", options);

            if (results.isEmpty()) {
                System.out.println("   'Search' not found on specified pages");
            } else {
                System.out.printf("   Found %d results on pages 1-%d:%n",
                        results.size(), Math.min(2, pageCount));
                results.forEach(r -> System.out.printf("   - Page %d%n", r.getPage() + 1));
            }
        }
    }

    /**
     * Truncate text for display
     */
    private static String truncate(String text, int maxLength) {
        if (text.length() <= maxLength) {
            return text;
        }
        return text.substring(0, maxLength - 3) + "...";
    }
}
