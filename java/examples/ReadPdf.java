import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.conversion.ConversionOptions;

import java.util.Arrays;

/**
 * Example: Read and extract text from a PDF document.
 *
 * This example demonstrates:
 * - Opening a PDF file
 * - Extracting document metadata (version, page count)
 * - Extracting text from pages
 * - Converting pages to different formats (Markdown, HTML, PlainText)
 *
 * Usage: java ReadPdf <pdf-file>
 */
public class ReadPdf {

    public static void main(String[] args) {
        if (args.length < 1) {
            System.err.println("Usage: java ReadPdf <pdf-file>");
            System.err.println("");
            System.err.println("Example: java ReadPdf document.pdf");
            System.exit(1);
        }

        String pdfPath = args[0];

        try (PdfDocument doc = PdfDocument.open(pdfPath)) {
            // Print document information
            printDocumentInfo(doc);

            // Extract text from first page
            System.out.println("\n" + "=".repeat(60));
            System.out.println("TEXT EXTRACTION (First Page)");
            System.out.println("=".repeat(60));
            extractAndPrintText(doc);

            // Convert first page to Markdown
            System.out.println("\n" + "=".repeat(60));
            System.out.println("MARKDOWN CONVERSION (First Page)");
            System.out.println("=".repeat(60));
            convertAndPrintMarkdown(doc);

            // Convert first page to HTML
            System.out.println("\n" + "=".repeat(60));
            System.out.println("HTML CONVERSION (First Page)");
            System.out.println("=".repeat(60));
            convertAndPrintHtml(doc);

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    /**
     * Print basic document information
     */
    private static void printDocumentInfo(PdfDocument doc) throws Exception {
        System.out.println("PDF Document Information");
        System.out.println("-".repeat(60));

        // Get PDF version
        int[] version = doc.getVersion();
        System.out.printf("PDF Version: %d.%d%n", version[0], version[1]);

        // Get page count
        int pageCount = doc.getPageCount();
        System.out.printf("Total Pages: %d%n", pageCount);

        // Check for structure tree (Tagged PDF)
        boolean hasStructure = doc.hasStructureTree();
        System.out.printf("Tagged PDF: %s%n", hasStructure ? "Yes" : "No");
    }

    /**
     * Extract and print text from the first page
     */
    private static void extractAndPrintText(PdfDocument doc) throws Exception {
        int pageCount = doc.getPageCount();
        if (pageCount == 0) {
            System.out.println("Document has no pages!");
            return;
        }

        try {
            String text = doc.extractText(0);
            System.out.println("Extracted Text:");
            System.out.println("-".repeat(60));
            System.out.println(text);
            System.out.println("-".repeat(60));
            System.out.printf("Character count: %d%n", text.length());
        } catch (Exception e) {
            System.err.println("Failed to extract text: " + e.getMessage());
        }
    }

    /**
     * Convert first page to Markdown and print
     */
    private static void convertAndPrintMarkdown(PdfDocument doc) throws Exception {
        int pageCount = doc.getPageCount();
        if (pageCount == 0) {
            System.out.println("Document has no pages!");
            return;
        }

        try {
            ConversionOptions options = ConversionOptions.builder()
                    .detectHeadings(true)
                    .preserveLayout(false)
                    .build();

            String markdown = doc.toMarkdown(0, options);
            System.out.println("Converted Markdown:");
            System.out.println("-".repeat(60));
            System.out.println(markdown);
            System.out.println("-".repeat(60));
            System.out.printf("Markdown length: %d characters%n", markdown.length());
        } catch (Exception e) {
            System.err.println("Failed to convert to Markdown: " + e.getMessage());
        }
    }

    /**
     * Convert first page to HTML and print preview
     */
    private static void convertAndPrintHtml(PdfDocument doc) throws Exception {
        int pageCount = doc.getPageCount();
        if (pageCount == 0) {
            System.out.println("Document has no pages!");
            return;
        }

        try {
            ConversionOptions options = ConversionOptions.builder().build();

            String html = doc.toHtml(0, options);
            System.out.println("Converted HTML (first 500 chars):");
            System.out.println("-".repeat(60));

            // Print first 500 characters
            String preview = html.length() > 500 ? html.substring(0, 500) + "..." : html;
            System.out.println(preview);
            System.out.println("-".repeat(60));
            System.out.printf("Total HTML length: %d characters%n", html.length());
        } catch (Exception e) {
            System.err.println("Failed to convert to HTML: " + e.getMessage());
        }
    }
}
