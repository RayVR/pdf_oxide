package com.pdfoxide.dom;

import com.pdfoxide.geometry.Rect;

/**
 * Strongly-typed structure element in a PDF page.
 *
 * <p>Structure elements represent the logical structure of a Tagged PDF document,
 * providing semantic information about the content hierarchy.
 *
 * @since 1.0.0
 */
public final class PdfStructure extends PdfElement {
    private final ElementId id;
    private final String structureType;
    private final Rect bbox;

    PdfStructure(ElementId id, String structureType, Rect bbox) {
        this.id = id;
        this.structureType = structureType;
        this.bbox = bbox;
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
     * Gets the structure type (e.g., "Document", "Paragraph", "Heading").
     *
     * @return structure type string
     */
    public String getStructureType() {
        return structureType;
    }

    /**
     * Common structure type: Document root.
     */
    public static final String TYPE_DOCUMENT = "Document";

    /**
     * Common structure type: Paragraph.
     */
    public static final String TYPE_PARAGRAPH = "Paragraph";

    /**
     * Common structure type: Heading.
     */
    public static final String TYPE_HEADING = "Heading";

    /**
     * Common structure type: List.
     */
    public static final String TYPE_LIST = "List";

    /**
     * Common structure type: Table.
     */
    public static final String TYPE_TABLE = "Table";

    /**
     * Common structure type: Figure.
     */
    public static final String TYPE_FIGURE = "Figure";

    @Override
    public String toString() {
        return String.format("PdfStructure(id=%s, type=%s)", id, structureType);
    }
}
