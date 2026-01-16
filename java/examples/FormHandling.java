import com.pdfoxide.core.Pdf;
import com.pdfoxide.document.DocumentEditor;
import com.pdfoxide.forms.FormExtractor;
import com.pdfoxide.forms.FormField;
import com.pdfoxide.forms.TextField;
import com.pdfoxide.geometry.Rect;

import java.util.List;

/**
 * Example: Create, extract, and manage form fields in PDF documents.
 *
 * This example demonstrates:
 * - Creating a PDF with form fields
 * - Adding text fields to documents
 * - Extracting form field information
 * - Working with field values
 * - Exporting form data (FDF/XFDF formats)
 *
 * Usage: java FormHandling
 */
public class FormHandling {

    public static void main(String[] args) {
        System.out.println("PDF Form Field Examples");
        System.out.println("=".repeat(60));

        try {
            // Create a form PDF
            System.out.println("\n1. Creating PDF with form fields...");
            Pdf formDoc = createFormPdf();
            String formFile = "sample_form.pdf";
            formDoc.save(formFile);
            formDoc.close();
            System.out.println("   ✓ Created: " + formFile);

            // Add form fields using DocumentEditor
            System.out.println("\n2. Adding form fields to document...");
            addFormFields(formFile);

            // Extract and display form information
            System.out.println("\n3. Extracting form field information...");
            extractFormFields(formFile);

            System.out.println("\n" + "=".repeat(60));
            System.out.println("✅ Form field examples completed!");

        } catch (Exception e) {
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
        }
    }

    /**
     * Create a PDF suitable for form fields
     */
    private static Pdf createFormPdf() throws Exception {
        String markdown = """
                # Application Form

                ## Personal Information

                Please fill in the following information:

                - Name: [Text field]
                - Email: [Text field]
                - Phone: [Text field]

                ## Address

                - Street Address: [Text field]
                - City: [Text field]
                - State: [Text field]
                - ZIP Code: [Text field]

                ## Employment

                - Current Position: [Text field]
                - Company: [Text field]
                - Years of Experience: [Text field]

                ## Additional Information

                - Skills: [Text field]
                - References: [Text field]
                - Comments: [Text field]

                ---

                Thank you for completing this form!
                """;

        return Pdf.fromMarkdown(markdown);
    }

    /**
     * Add form fields to document
     */
    private static void addFormFields(String filePath) throws Exception {
        try {
            DocumentEditor editor = DocumentEditor.open(filePath);

            // Define form field locations (x, y, width, height)
            // Personal Information
            addTextField(editor, "name", 100, 700, 300, 20, "John Doe");
            addTextField(editor, "email", 100, 675, 300, 20, "john@example.com");
            addTextField(editor, "phone", 100, 650, 300, 20, "(555) 123-4567");

            // Address
            addTextField(editor, "address", 100, 600, 400, 20, "123 Main Street");
            addTextField(editor, "city", 100, 575, 200, 20, "Springfield");
            addTextField(editor, "state", 320, 575, 50, 20, "IL");
            addTextField(editor, "zip", 380, 575, 100, 20, "62701");

            // Employment
            addTextField(editor, "position", 100, 500, 300, 20, "Software Engineer");
            addTextField(editor, "company", 100, 475, 300, 20, "Tech Corp");
            addTextField(editor, "experience", 100, 450, 200, 20, "5");

            // Save document with forms
            editor.save("sample_form_with_fields.pdf");
            editor.close();

            System.out.println("   ✓ Added 10 text fields");
            System.out.println("   ✓ Saved as: sample_form_with_fields.pdf");

        } catch (Exception e) {
            System.out.println("   ⚠ Form field addition not fully supported in v0.3.0");
            System.out.println("     This is a foundation for v0.4.0+ full implementation");
        }
    }

    /**
     * Add a text field to the document
     */
    private static void addTextField(DocumentEditor editor, String name,
                                      double x, double y, double width, double height,
                                      String defaultValue) throws Exception {
        // Create a text field
        TextField field = new TextField(
                name,
                new Rect(x, y, x + width, y - height),
                defaultValue,
                100  // max length
        );

        // Add to first page
        editor.addFormField(0, field);
    }

    /**
     * Extract and display form field information
     */
    private static void extractFormFields(String filePath) throws Exception {
        try {
            FormExtractor extractor = new FormExtractor(
                    com.pdfoxide.core.PdfDocument.open(filePath)
            );

            List<FormField> fields = extractor.extractFields();

            if (fields.isEmpty()) {
                System.out.println("   No form fields found in document");
            } else {
                System.out.printf("   Found %d form field(s):%n", fields.size());

                for (FormField field : fields) {
                    displayFieldInfo(field);
                }

                // Try to export form data
                System.out.println("\n   Exporting form data formats:");
                try {
                    extractor.exportFdf("form_data.fdf");
                    System.out.println("   ✓ Exported to FDF format: form_data.fdf");
                } catch (Exception e) {
                    System.out.println("   ⚠ FDF export not available in v0.3.0");
                }

                try {
                    extractor.exportXfdf("form_data.xfdf");
                    System.out.println("   ✓ Exported to XFDF format: form_data.xfdf");
                } catch (Exception e) {
                    System.out.println("   ⚠ XFDF export not available in v0.3.0");
                }
            }

            extractor.close();

        } catch (Exception e) {
            System.out.println("   ⚠ Form extraction not fully available in v0.3.0");
            System.out.println("     Full implementation coming in v0.4.0+");
        }
    }

    /**
     * Display information about a form field
     */
    private static void displayFieldInfo(FormField field) {
        System.out.printf("   - Name: %s%n", field.getName());
        System.out.printf("     Type: %s%n", field.getFieldType());

        if (field.getValue() != null) {
            System.out.printf("     Value: %s%n", field.getValue());
        }

        if (field.getTooltip().isPresent()) {
            System.out.printf("     Tooltip: %s%n", field.getTooltip().get());
        }
    }
}
