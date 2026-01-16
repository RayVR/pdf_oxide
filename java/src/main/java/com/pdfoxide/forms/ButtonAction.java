package com.pdfoxide.forms;

/**
 * Actions for push buttons.
 */
public final class ButtonAction {
    private final String type;
    private final String target;

    private ButtonAction(String type, String target) {
        this.type = type;
        this.target = target;
    }

    /**
     * Creates a submit action.
     *
     * @param url submit URL
     * @return button action
     */
    public static ButtonAction submit(String url) {
        return new ButtonAction("submit", url);
    }

    /**
     * Creates a reset action.
     *
     * @return button action
     */
    public static ButtonAction reset() {
        return new ButtonAction("reset", "");
    }

    /**
     * Creates a custom action.
     *
     * @param type action type
     * @param target action target
     * @return button action
     */
    public static ButtonAction custom(String type, String target) {
        return new ButtonAction(type, target);
    }

    public String getType() {
        return type;
    }

    public String getTarget() {
        return target;
    }
}
