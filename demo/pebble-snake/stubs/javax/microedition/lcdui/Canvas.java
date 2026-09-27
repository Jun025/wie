package javax.microedition.lcdui;

public abstract class Canvas extends Displayable {
    public static final int UP = 1, DOWN = 6, LEFT = 2, RIGHT = 5, FIRE = 8;
    public static final int KEY_NUM0 = 48, KEY_NUM2 = 50, KEY_NUM4 = 52, KEY_NUM5 = 53, KEY_NUM6 = 54, KEY_NUM8 = 56;
    protected Canvas() {}
    protected abstract void paint(Graphics g);
    public final void repaint() {}
    public final void serviceRepaints() {}
    public int getGameAction(int keyCode) { return 0; }
    protected void keyPressed(int keyCode) {}
    public void setFullScreenMode(boolean mode) {}
}
