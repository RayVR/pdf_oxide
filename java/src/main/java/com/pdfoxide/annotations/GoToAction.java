package com.pdfoxide.annotations;

/**
 * GoTo action for annotations.
 */
public final class GoToAction extends AnnotationAction {
    private final int pageIndex;
    private final double x;
    private final double y;
    private final double zoom;

    public GoToAction(int pageIndex, double x, double y, double zoom) {
        this.pageIndex = pageIndex;
        this.x = x;
        this.y = y;
        this.zoom = zoom;
    }

    public int getPageIndex() {
        return pageIndex;
    }

    public double getX() {
        return x;
    }

    public double getY() {
        return y;
    }

    public double getZoom() {
        return zoom;
    }

    @Override
    public String getActionType() {
        return "GoTo";
    }
}
