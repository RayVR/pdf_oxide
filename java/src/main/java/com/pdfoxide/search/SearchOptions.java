package com.pdfoxide.search;

/**
 * Options for text search operations.
 */
public final class SearchOptions {
    private final boolean caseSensitive;
    private final boolean wholeWord;
    private final boolean useRegex;
    private final Integer maxResults;
    private final Integer pageIndex;

    private SearchOptions(Builder builder) {
        this.caseSensitive = builder.caseSensitive;
        this.wholeWord = builder.wholeWord;
        this.useRegex = builder.useRegex;
        this.maxResults = builder.maxResults;
        this.pageIndex = builder.pageIndex;
    }

    /**
     * Creates a new builder.
     *
     * @return new builder
     */
    public static Builder builder() {
        return new Builder();
    }

    public boolean isCaseSensitive() {
        return caseSensitive;
    }

    public boolean isWholeWord() {
        return wholeWord;
    }

    public boolean isUseRegex() {
        return useRegex;
    }

    public Integer getMaxResults() {
        return maxResults;
    }

    public Integer getPageIndex() {
        return pageIndex;
    }

    /**
     * Builder for SearchOptions.
     */
    public static final class Builder {
        private boolean caseSensitive = false;
        private boolean wholeWord = false;
        private boolean useRegex = false;
        private Integer maxResults;
        private Integer pageIndex;

        public Builder caseSensitive(boolean caseSensitive) {
            this.caseSensitive = caseSensitive;
            return this;
        }

        public Builder wholeWord(boolean wholeWord) {
            this.wholeWord = wholeWord;
            return this;
        }

        public Builder useRegex(boolean useRegex) {
            this.useRegex = useRegex;
            return this;
        }

        public Builder maxResults(int maxResults) {
            this.maxResults = maxResults;
            return this;
        }

        public Builder pageIndex(int pageIndex) {
            this.pageIndex = pageIndex;
            return this;
        }

        public SearchOptions build() {
            return new SearchOptions(this);
        }
    }
}
