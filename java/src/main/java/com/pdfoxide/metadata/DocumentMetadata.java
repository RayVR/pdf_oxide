package com.pdfoxide.metadata;

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
