package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;
import java.util.ArrayList;
import java.util.List;

/**
 * Strongly-typed table element in a PDF page.
 *
 * <p>Table elements represent tabular data with support for headers, captions,
 * and cell access/modification.
 *
 * @since 1.0.0
 */
public final class PdfTable extends PdfElement {
    private final ElementId id;
    private final Rect bbox;
    private final int rows;
    private final int cols;
    private final boolean hasHeader;
    private String caption;
    private final float detectionConfidence;
    private final boolean fromStructureTree;
    private final List<List<String>> cells;

    PdfTable(ElementId id, Rect bbox, int rows, int cols, boolean hasHeader,
            String caption, float detectionConfidence, boolean fromStructureTree,
            List<List<String>> cells) {
        this.id = id;
        this.bbox = bbox;
        this.rows = rows;
        this.cols = cols;
        this.hasHeader = hasHeader;
        this.caption = caption;
        this.detectionConfidence = detectionConfidence;
        this.fromStructureTree = fromStructureTree;
        this.cells = new ArrayList<>(cells);
    }

    @Override
    public ElementId getId() {
        return id;
    }

    @Override
    public Rect getBbox() {
        return bbox;
    }

    /**
     * Gets the number of rows.
     *
     * @return row count
     */
    public int getRowCount() {
        return rows;
    }

    /**
     * Gets the number of columns.
     *
     * @return column count
     */
    public int getColumnCount() {
        return cols;
    }

    /**
     * Checks if the table has a header row.
     *
     * @return true if first row is a header
     */
    public boolean hasHeader() {
        return hasHeader;
    }

    /**
     * Gets a cell value at the specified row and column.
     *
     * @param row row index (0-based)
     * @param col column index (0-based)
     * @return cell text or null if out of bounds
     */
    public String getCell(int row, int col) {
        if (row < 0 || row >= rows || col < 0 || col >= cols) {
            return null;
        }
        List<String> rowCells = cells.get(row);
        if (rowCells == null || col >= rowCells.size()) {
            return null;
        }
        return rowCells.get(col);
    }

    /**
     * Sets a cell value at the specified row and column.
     *
     * @param row row index (0-based)
     * @param col column index (0-based)
     * @param text new cell text
     * @return true if cell was updated
     */
    public boolean setCell(int row, int col, String text) {
        if (row < 0 || row >= rows || col < 0 || col >= cols) {
            return false;
        }
        List<String> rowCells = cells.get(row);
        if (rowCells == null || col >= rowCells.size()) {
            return false;
        }
        rowCells.set(col, text);
        return true;
    }

    /**
     * Gets the table caption.
     *
     * @return caption text or null if not set
     */
    public String getCaption() {
        return caption;
    }

    /**
     * Sets the table caption.
     *
     * @param caption new caption text
     */
    public void setCaption(String caption) {
        this.caption = caption;
    }

    /**
     * Gets the detection confidence (if table was detected via heuristics).
     *
     * @return confidence value (0.0-1.0)
     */
    public float getDetectionConfidence() {
        return detectionConfidence;
    }

    /**
     * Checks if this table came from the PDF structure tree (Tagged PDF).
     *
     * @return true if from structure tree
     */
    public boolean isFromStructureTree() {
        return fromStructureTree;
    }

    /**
     * Gets all rows as lists of cell text.
     *
     * @return list of rows, where each row is a list of cell strings
     */
    public List<List<String>> getRows() {
        return new ArrayList<>(cells);
    }

    /**
     * Gets a specific row.
     *
     * @param row row index (0-based)
     * @return list of cell strings in that row, or null if out of bounds
     */
    public List<String> getRow(int row) {
        if (row < 0 || row >= rows) {
            return null;
        }
        List<String> rowCells = cells.get(row);
        return rowCells != null ? new ArrayList<>(rowCells) : null;
    }

    /**
     * Converts this table to an HTML representation.
     *
     * @return HTML table element
     */
    public String toHtml() {
        StringBuilder html = new StringBuilder("<table border=\"1\">\n");

        if (caption != null && !caption.isEmpty()) {
            html.append("  <caption>").append(escapeHtml(caption)).append("</caption>\n");
        }

        if (hasHeader && rows > 0) {
            html.append("  <thead>\n    <tr>\n");
            List<String> headerRow = cells.get(0);
            if (headerRow != null) {
                for (String cell : headerRow) {
                    html.append("      <th>").append(escapeHtml(cell)).append("</th>\n");
                }
            }
            html.append("    </tr>\n  </thead>\n");
        }

        html.append("  <tbody>\n");
        int startRow = hasHeader ? 1 : 0;
        for (int i = startRow; i < rows; i++) {
            html.append("    <tr>\n");
            List<String> rowCells = cells.get(i);
            if (rowCells != null) {
                for (String cell : rowCells) {
                    html.append("      <td>").append(escapeHtml(cell)).append("</td>\n");
                }
            }
            html.append("    </tr>\n");
        }
        html.append("  </tbody>\n");
        html.append("</table>");

        return html.toString();
    }

    private static String escapeHtml(String text) {
        if (text == null) {
            return "";
        }
        return text.replace("&", "&amp;")
                   .replace("<", "&lt;")
                   .replace(">", "&gt;")
                   .replace("\"", "&quot;")
                   .replace("'", "&#39;");
    }

    @Override
    public String toString() {
        return String.format("PdfTable(id=%s, %dx%d, header=%s)",
                             id, rows, cols, hasHeader);
    }
}
