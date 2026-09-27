/*
 * Pebble Snake — a small MIDP demo game written for the wie featurephone
 * "try it now" button. Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 */

import javax.microedition.lcdui.Display;
import javax.microedition.midlet.MIDlet;

public class PebbleSnake extends MIDlet {
    private SnakeCanvas canvas;

    protected void startApp() {
        if (canvas == null) {
            canvas = new SnakeCanvas();
            Display.getDisplay(this).setCurrent(canvas);
            new Thread(canvas).start();
        }
    }

    protected void pauseApp() {}

    protected void destroyApp(boolean unconditional) {
        if (canvas != null) {
            canvas.quit();
        }
    }
}
