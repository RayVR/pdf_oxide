package com.pdfoxide.annotations;

/**
 * Launch action for annotations.
 */
public final class LaunchAction extends AnnotationAction {
    private final String filePath;
    private final String parameters;

    public LaunchAction(String filePath, String parameters) {
        this.filePath = filePath;
        this.parameters = parameters;
    }

    public String getFilePath() {
        return filePath;
    }

    public String getParameters() {
        return parameters;
    }

    @Override
    public String getActionType() {
        return "Launch";
    }
}
