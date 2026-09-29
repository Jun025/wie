package javax.microedition.lcdui;

public class Graphics {
    public static final int HCENTER = 1, VCENTER = 2, LEFT = 4, RIGHT = 8, TOP = 16, BOTTOM = 32, BASELINE = 64;
    public void setColor(int rgb) {}
    public void setFont(Font f) {}
    public void fillRect(int x, int y, int w, int h) {}
    public void drawRect(int x, int y, int w, int h) {}
    public void fillRoundRect(int x, int y, int w, int h, int aw, int ah) {}
    public void fillArc(int x, int y, int w, int h, int s, int a) {}
    public void drawRoundRect(int x, int y, int w, int h, int aw, int ah) {}
    public void drawLine(int x1, int y1, int x2, int y2) {}
    public void drawString(String s, int x, int y, int anchor) {}
    public void drawImage(Image img, int x, int y, int anchor) {}
}
