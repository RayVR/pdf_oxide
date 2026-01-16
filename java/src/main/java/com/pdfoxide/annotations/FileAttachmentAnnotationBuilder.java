package com.pdfoxide.annotations;

import com.pdfoxide.geometry.Rect;

/**
 * Builder for file attachment annotations.
 */
public final class FileAttachmentAnnotationBuilder {
    private final Rect rect;
    private final String filePath;
    private String fileName;
    private FileAttachmentIcon icon = FileAttachmentIcon.PUSH_PIN;

    private FileAttachmentAnnotationBuilder(Rect rect, String filePath) {
        this.rect = rect;
        this.filePath = filePath;
    }

    public static FileAttachmentAnnotationBuilder create(Rect rect, String filePath) {
        return new FileAttachmentAnnotationBuilder(rect, filePath);
    }

    public FileAttachmentAnnotationBuilder fileName(String name) {
        this.fileName = name;
        return this;
    }

    public FileAttachmentAnnotationBuilder icon(FileAttachmentIcon icon) {
        this.icon = icon;
        return this;
    }

    public FileAttachmentAnnotation build() {
        return new FileAttachmentAnnotation(rect, filePath, fileName, icon);
    }
}
