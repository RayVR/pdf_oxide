package com.pdfoxide.document;

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
                "title='" + title + '\'' +
                ", author='" + author + '\'' +
                ", subject='" + subject + '\'' +
                ", keywords='" + keywords + '\'' +
                ", pageCount=" + pageCount +
                '}';
    }
}
