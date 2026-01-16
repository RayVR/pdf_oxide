import com.pdfoxide.core.Pdf;
import com.pdfoxide.core.PdfDocument;
import com.pdfoxide.compliance.ComplianceError;
import com.pdfoxide.compliance.ComplianceWarning;
import com.pdfoxide.compliance.PdfALevel;
import com.pdfoxide.compliance.PdfAValidator;
import com.pdfoxide.compliance.ValidationResult;

import java.util.List;

/**
 * Example: Validate PDF/A compliance.
 *
 * This example demonstrates:
 * - Creating a PDF document
 * - Validating against different PDF/A levels
 * - Analyzing validation results
 * - Reviewing errors and warnings
 * - Generating compliance reports
 *
 * PDF/A is an ISO standard for long-term archival of PDF documents.
 *
 * Usage: java ValidatePdfa
 */
public class ValidatePdfa {

    public static void main(String[] args) {
        System.out.println("PDF/A Compliance Validation Examples");
        System.out.println("=".repeat(70));

        try {
            // Create a sample PDF
            System.out.println("\n1. Creating sample PDF...");
            Pdf sampleDoc = createSamplePdf();
            String sampleFile = "sample_compliance.pdf";
            sampleDoc.save(sampleFile);
            sampleDoc.close();
            System.out.println("   ✓ Created: " + sampleFile);

            // Validate against different PDF/A levels
            try (PdfDocument doc = PdfDocument.open(sampleFile)) {
                System.out.println("\n2. Validating against PDF/A levels...\n");
                System.out.println("-".repeat(70));

                // Validate Level 1B (baseline)
                validateLevel(doc, PdfALevel.LEVEL_1B);

                // Validate Level 2B
                validateLevel(doc, PdfALevel.LEVEL_2B);

                // Validate Level 3B
                validateLevel(doc, PdfALevel.LEVEL_3B);
            }

            System.out.println("\n" + "=".repeat(70));
            System.out.println("✅ PDF/A validation examples completed!");

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }

    /**
     * Create a sample PDF for compliance testing
     */
    private static Pdf createSamplePdf() throws Exception {
        String markdown = """
                # PDF/A Compliance Test Document

                ## Document Information

                This PDF demonstrates compliance validation against the PDF/A standard.
                PDF/A is designed for long-term archival and preservation of digital documents.

                ## Standards

                PDF/A has multiple levels:
                - **Level 1**: Based on PDF 1.4
                - **Level 2**: Based on PDF 1.7
                - **Level 3**: Based on PDF 1.7 with Extensions

                Each level has variants:
                - **Variant A**: Tagged PDF with all semantic structures
                - **Variant B**: Baseline profile without color space restrictions
                - **Variant U**: Unicode mapping support (Level 2-3 only)

                ## Content Requirements

                PDF/A documents must:
                - Be completely self-contained
                - Embed all required fonts
                - Avoid JavaScript and other executable content
                - Use standard color spaces
                - Have predictable rendering

                ## Testing Compliance

                Use validators to check conformance:
                - Check for embedded fonts
                - Verify color spaces
                - Validate document structure
                - Ensure no external dependencies

                ## Conclusion

                PDF/A compliance ensures documents remain accessible and renderable
                for decades without dependency on specific software or hardware.
                """;

        return Pdf.fromMarkdown(markdown);
    }

    /**
     * Validate a document against a specific PDF/A level
     */
    private static void validateLevel(PdfDocument doc, PdfALevel level) throws Exception {
        System.out.printf("A. Validating against %s%n", getReadableLevel(level));
        System.out.println("-".repeat(70));

        try {
            PdfAValidator validator = new PdfAValidator(level);
            ValidationResult result = validator.validate(doc);

            // Print basic status
            System.out.printf("Status: %s%n", result.isValid() ? "✓ VALID" : "✗ INVALID");
            System.out.printf("Errors: %d%n", result.getErrors().size());
            System.out.printf("Warnings: %d%n", result.getWarnings().size());

            // Print errors
            if (!result.getErrors().isEmpty()) {
                System.out.println("\nErrors:");
                List<ComplianceError> errors = result.getErrors();
                for (int i = 0; i < Math.min(errors.size(), 3); i++) {
                    ComplianceError error = errors.get(i);
                    System.out.printf("  [%d] %s: %s%n",
                            i + 1,
                            error.getCode().toString(),
                            error.getMessage());
                }
                if (errors.size() > 3) {
                    System.out.printf("  ... and %d more errors%n", errors.size() - 3);
                }
            }

            // Print warnings
            if (!result.getWarnings().isEmpty()) {
                System.out.println("\nWarnings:");
                List<ComplianceWarning> warnings = result.getWarnings();
                for (int i = 0; i < Math.min(warnings.size(), 3); i++) {
                    ComplianceWarning warning = warnings.get(i);
                    System.out.printf("  [%d] %s (Severity %d): %s%n",
                            i + 1,
                            warning.getCode().toString(),
                            warning.getSeverity(),
                            warning.getMessage());
                }
                if (warnings.size() > 3) {
                    System.out.printf("  ... and %d more warnings%n", warnings.size() - 3);
                }
            }

            // Print statistics
            if (result.getStats() != null) {
                System.out.println("\nValidation Statistics:");
                System.out.printf("  Pages Checked: %d%n", result.getStats().getPagesChecked());
                System.out.printf("  Elements Analyzed: %d%n", result.getStats().getElementsAnalyzed());
                System.out.printf("  Fonts Validated: %d%n", result.getStats().getFontsValidated());
                System.out.printf("  Validation Time: %d ms%n", result.getStats().getValidationTime());
            }

        } catch (Exception e) {
            System.err.printf("Validation failed: %s%n", e.getMessage());
        }

        System.out.println();
    }

    /**
     * Convert PDF/A level enum to readable format
     */
    private static String getReadableLevel(PdfALevel level) {
        return switch (level) {
            case LEVEL_1A -> "PDF/A-1a (Level 1, Variant A)";
            case LEVEL_1B -> "PDF/A-1b (Level 1, Variant B)";
            case LEVEL_2A -> "PDF/A-2a (Level 2, Variant A)";
            case LEVEL_2B -> "PDF/A-2b (Level 2, Variant B)";
            case LEVEL_2U -> "PDF/A-2u (Level 2, Variant U)";
            case LEVEL_3A -> "PDF/A-3a (Level 3, Variant A)";
            case LEVEL_3B -> "PDF/A-3b (Level 3, Variant B)";
            case LEVEL_3U -> "PDF/A-3u (Level 3, Variant U)";
        };
    }
}
