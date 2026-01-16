package com.pdfoxide.compliance;

/**
 * Statistics from a PDF/A validation run.
 *
 * @since 1.0.0
 */
public final class ValidationStats {
    private final long validationTime;
    private final int pagesChecked;
    private final int elementsAnalyzed;
    private final int annotationsChecked;
    private final int imagesAnalyzed;
    private final int fontsValidated;

    /**
     * Constructs validation statistics.
     *
     * @param validationTime time spent validating in milliseconds
     * @param pagesChecked number of pages checked
     * @param elementsAnalyzed number of document elements analyzed
     * @param annotationsChecked number of annotations checked
     * @param imagesAnalyzed number of images analyzed
     * @param fontsValidated number of fonts validated
     */
    public ValidationStats(
            long validationTime,
            int pagesChecked,
            int elementsAnalyzed,
            int annotationsChecked,
            int imagesAnalyzed,
            int fontsValidated) {
        this.validationTime = validationTime;
        this.pagesChecked = pagesChecked;
        this.elementsAnalyzed = elementsAnalyzed;
        this.annotationsChecked = annotationsChecked;
        this.imagesAnalyzed = imagesAnalyzed;
        this.fontsValidated = fontsValidated;
    }

    /**
     * Gets the validation time in milliseconds.
     *
     * @return validation duration
     */
    public long getValidationTime() {
        return validationTime;
    }

    /**
     * Gets the number of pages checked.
     *
     * @return page count
     */
    public int getPagesChecked() {
        return pagesChecked;
    }

    /**
     * Gets the number of document elements analyzed.
     *
     * @return element count
     */
    public int getElementsAnalyzed() {
        return elementsAnalyzed;
    }

    /**
     * Gets the number of annotations checked.
     *
     * @return annotation count
     */
    public int getAnnotationsChecked() {
        return annotationsChecked;
    }

    /**
     * Gets the number of images analyzed.
     *
     * @return image count
     */
    public int getImagesAnalyzed() {
        return imagesAnalyzed;
    }

    /**
     * Gets the number of fonts validated.
     *
     * @return font count
     */
    public int getFontsValidated() {
        return fontsValidated;
    }

    /**
     * Gets the average elements per page.
     *
     * @return average element count
     */
    public double getAverageElementsPerPage() {
        return pagesChecked > 0 ? (double) elementsAnalyzed / pagesChecked : 0.0;
    }

    /**
     * Gets the average validation time per page.
     *
     * @return time in milliseconds
     */
    public double getAverageTimePerPage() {
        return pagesChecked > 0 ? (double) validationTime / pagesChecked : 0.0;
    }

    @Override
    public String toString() {
        return String.format(
            "ValidationStats(time=%dms, pages=%d, elements=%d, annotations=%d, images=%d, fonts=%d)",
            validationTime,
            pagesChecked,
            elementsAnalyzed,
            annotationsChecked,
            imagesAnalyzed,
            fontsValidated
        );
    }
}
