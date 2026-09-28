/*
 * otterpebble arcade — Copyright (c) 2026 otterpebble. MIT License (see LICENSE).
 *
 * The frame every arcade demo shares: title screen (= the key guide), the play loop, the
 * top bar, the game-over card, 0 = sound, and the best score kept in a RecordStore.
 * A game only fills in the rules and the board. Colours follow otterpebble DESIGN.md:
 * a light neutral surface, one text colour, and colour only where it means something
 * (each game brings its own 3~5 pastel set for its pieces).
 */

import java.util.Random;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;

abstract class Arcade extends Canvas implements Runnable {
    static final int TITLE = 0, PLAY = 1, OVER = 2;

    // DESIGN.md light tokens: --background, --surface, --surface-2, --text, --text-sub.
    static final int WHITE = 0xFFFFFF;
    static final int SURFACE = 0xF4F5F7;
    static final int SURFACE2 = 0xEBEDF0;
    static final int TEXT = 0x191F28;
    static final int SUB = 0x616B78;

    final Sound sound = new Sound();
    final Random random = new Random();
    final int lineH;

    volatile boolean running = true;
    int state = TITLE;
    int score, best;
    boolean cleared, newBest;
    int frame;
    long overAt;
    String toast;
    long toastUntil;

    Arcade() {
        setFullScreenMode(true);
        lineH = Font.getDefaultFont().getHeight();
        best = loadBest();
    }

    // ---- what a game supplies ----
    abstract String name();

    abstract String tagline();

    abstract String[] guide();

    abstract void newGame();

    /** A key during play. {@code action} is the game action with 2/4/6/8/5 folded in. */
    abstract void key(int action, int key);

    /** Called every loop turn while playing; for timers and falling pieces. */
    void tick(long now) {}

    /** Milliseconds between loop turns while playing. */
    int delay() {
        return 150;
    }

    /** The middle of the top bar: moves left, time left, … */
    String status() {
        return "";
    }

    abstract void paintBoard(Graphics g, int top, int w, int h);

    /** A small picture for the title screen, centred on (cx, cy). */
    abstract void paintEmblem(Graphics g, int cx, int cy);

    // ---- helpers for games ----
    int rnd(int n) {
        return (random.nextInt() >>> 1) % n;
    }

    void gameOver(boolean clear) {
        state = OVER;
        cleared = clear;
        overAt = System.currentTimeMillis();
        newBest = score > best;
        if (newBest) {
            best = score;
            saveBest(best);
        }
        sound.play(clear ? Sound.BIG : Sound.OVER);
    }

    void toast(String text) {
        toast = text;
        toastUntil = System.currentTimeMillis() + 1200;
    }

    /** A rounded tile; {@code ring} draws the selection outline around it. */
    static void tile(Graphics g, int x, int y, int s, int color, boolean ring) {
        int r = s / 3;
        if (ring) {
            g.setColor(TEXT);
            g.fillRoundRect(x - 3, y - 3, s + 6, s + 6, r + 6, r + 6);
            g.setColor(WHITE);
            g.fillRoundRect(x - 1, y - 1, s + 2, s + 2, r + 2, r + 2);
        }
        g.setColor(color);
        g.fillRoundRect(x, y, s, s, r, r);
    }

    /** The host font has one size, so emphasis is weight: the string drawn twice, 1px apart. */
    static void bold(Graphics g, String s, int x, int y, int anchor) {
        g.drawString(s, x, y, anchor);
        g.drawString(s, x + 1, y, anchor);
    }

    // ---- the frame ----
    public void run() {
        while (running) {
            if (state == PLAY) {
                synchronized (this) {
                    tick(System.currentTimeMillis());
                }
            }
            frame++;
            repaint();
            try {
                Thread.sleep(state == PLAY ? delay() : 200);
            } catch (InterruptedException e) {
                return;
            }
        }
    }

    void quit() {
        running = false;
        sound.stop();
    }

    protected synchronized void keyPressed(int key) {
        if (key == KEY_NUM0) {
            sound.enabled = !sound.enabled;
            if (sound.enabled) sound.play(Sound.TOGGLE);
            else sound.stop();
            toast(sound.enabled ? "소리 켬" : "소리 끔");
            repaint();
            return;
        }
        int action = getGameAction(key);
        // Keypad digits first: handsets differ in which digits they report as game actions.
        if (key == KEY_NUM2) action = UP;
        else if (key == KEY_NUM8) action = DOWN;
        else if (key == KEY_NUM4) action = LEFT;
        else if (key == KEY_NUM6) action = RIGHT;
        else if (key == KEY_NUM5) action = FIRE;

        if (state == PLAY) {
            key(action, key);
        } else if (action == FIRE) {
            // A short lock so the key that ended the game does not also start the next one.
            if (state == TITLE || System.currentTimeMillis() - overAt > 800) {
                score = 0;
                newGame();
                state = PLAY;
                sound.play(Sound.START);
            }
        }
        repaint();
    }

    protected void paint(Graphics g) {
        int w = getWidth();
        int h = getHeight();
        g.setColor(SURFACE);
        g.fillRect(0, 0, w, h);
        if (state == TITLE) {
            paintTitle(g, w, h);
        } else {
            int bar = lineH + 10;
            synchronized (this) {
                paintBoard(g, bar, w, h);
            }
            g.setColor(WHITE);
            g.fillRect(0, 0, w, bar);
            g.setColor(TEXT);
            bold(g, "점수 " + score, 6, 5, Graphics.TOP | Graphics.LEFT);
            g.setColor(SUB);
            g.drawString("최고 " + best, w - 6, 5, Graphics.TOP | Graphics.RIGHT);
            g.setColor(TEXT);
            g.drawString(status(), w / 2 + 6, 5, Graphics.TOP | Graphics.HCENTER);
            if (state == OVER) {
                paintOver(g, w, h);
            }
        }
        if (toast != null && System.currentTimeMillis() < toastUntil) {
            int tw = Font.getDefaultFont().stringWidth(toast) + 20;
            g.setColor(TEXT);
            g.fillRoundRect((w - tw) / 2, h - lineH - 22, tw, lineH + 10, 12, 12);
            g.setColor(WHITE);
            g.drawString(toast, w / 2, h - lineH - 17, Graphics.TOP | Graphics.HCENTER);
        }
    }

    private void paintTitle(Graphics g, int w, int h) {
        int cx = w / 2;
        int y = h / 6;
        paintEmblem(g, cx, y + 14);
        y += 44;
        g.setColor(TEXT);
        bold(g, name(), cx, y, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString(tagline(), cx, y + lineH + 6, Graphics.TOP | Graphics.HCENTER);

        String[] lines = guide();
        int step = lineH + 6;
        int cardH = lines.length * step + 16;
        int cardY = y + 2 * lineH + 20;
        g.setColor(WHITE);
        g.fillRoundRect(12, cardY, w - 24, cardH, 16, 16);
        g.setColor(TEXT);
        for (int i = 0; i < lines.length; i++) {
            g.drawString(lines[i], cx, cardY + 10 + i * step, Graphics.TOP | Graphics.HCENTER);
        }
        g.setColor(SUB);
        g.drawString("0 : 소리 " + (sound.enabled ? "켬" : "끔") + " · 최고 " + best, cx, cardY + cardH + 8,
            Graphics.TOP | Graphics.HCENTER);

        int by = cardY + cardH + lineH + 18;
        if ((frame / 3) % 4 != 3) {
            g.setColor(TEXT);
            g.fillRoundRect(cx - 70, by, 140, lineH + 12, 20, 20);
            g.setColor(WHITE);
            bold(g, "5 를 눌러 시작", cx, by + 6, Graphics.TOP | Graphics.HCENTER);
        }
        g.setColor(SUB);
        g.drawString("otterpebble", cx, h - lineH - 6, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintOver(Graphics g, int w, int h) {
        int cw = w - 48;
        int ch = 4 * lineH + 52;
        int x = 24, y = (h - ch) / 2;
        g.setColor(TEXT);
        g.fillRoundRect(x - 2, y - 2, cw + 4, ch + 4, 22, 22);
        g.setColor(WHITE);
        g.fillRoundRect(x, y, cw, ch, 20, 20);
        int cx = w / 2;
        g.setColor(TEXT);
        bold(g, cleared ? "완성!" : "게임 끝", cx, y + 10, Graphics.TOP | Graphics.HCENTER);
        g.drawString(score + " 점", cx, y + lineH + 18, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString(newBest ? "새 최고 기록!" : "최고 " + best, cx, y + 2 * lineH + 24, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SURFACE2);
        g.fillRoundRect(cx - 56, y + 3 * lineH + 32, 112, lineH + 10, 16, 16);
        g.setColor(TEXT);
        g.drawString("5 다시하기", cx, y + 3 * lineH + 37, Graphics.TOP | Graphics.HCENTER);
    }

    // ---- best score: one 4-byte record, named after the game ----
    private int loadBest() {
        try {
            RecordStore rs = RecordStore.openRecordStore(name(), true);
            int v = 0;
            if (rs.getNumRecords() > 0) {
                byte[] b = rs.getRecord(1);
                v = ((b[0] & 0xFF) << 24) | ((b[1] & 0xFF) << 16) | ((b[2] & 0xFF) << 8) | (b[3] & 0xFF);
            }
            rs.closeRecordStore();
            return v;
        } catch (Exception e) {
            return 0; // no storage: the best score lasts until the game is closed
        }
    }

    private void saveBest(int v) {
        try {
            RecordStore rs = RecordStore.openRecordStore(name(), true);
            byte[] b = {(byte) (v >>> 24), (byte) (v >>> 16), (byte) (v >>> 8), (byte) v};
            if (rs.getNumRecords() > 0) rs.setRecord(1, b, 0, 4);
            else rs.addRecord(b, 0, 4);
            rs.closeRecordStore();
        } catch (Exception e) {
            // keep playing without saving
        }
    }

    /** Every game's MIDlet: show the canvas, start its loop. */
    abstract static class Midlet extends MIDlet {
        private Arcade canvas;

        abstract Arcade create();

        protected void startApp() {
            if (canvas == null) {
                canvas = create();
                Display.getDisplay(this).setCurrent(canvas);
                new Thread(canvas).start();
            }
        }

        protected void pauseApp() {}

        protected void destroyApp(boolean unconditional) {
            if (canvas != null) canvas.quit();
        }
    }
}
