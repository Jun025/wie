/*
 * Pebble Snake — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * An otter-coloured snake collects pebbles. The title screen doubles as the key guide:
 * every control it lists (direction keys or 2/4/6/8, OK or 5, 0) is used in play.
 */

import java.util.Random;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;

final class SnakeCanvas extends Canvas implements Runnable {
    private static final int TITLE = 0, PLAY = 1, PAUSED = 2, OVER = 3;
    private static final int CELL = 10;
    private static final int HUD = 18;
    private static final int MAX = 400;

    private static final int WATER = 0x1D4E6B;
    private static final int WATER_DARK = 0x173F57;
    private static final int OTTER = 0x8A5A32;
    private static final int OTTER_HEAD = 0xB07A48;
    private static final int PEBBLE = 0xD9D2C3;
    private static final int PEBBLE_SHADE = 0x9A9282;
    private static final int TEXT = 0xFFFFFF;
    private static final int ACCENT = 0xFFD166;

    private final Sound sound = new Sound();
    private final Random random = new Random();
    private final int[] xs = new int[MAX];
    private final int[] ys = new int[MAX];

    private volatile boolean running = true;
    private int state = TITLE;
    private int cols, rows, originX, originY;
    private int length, dir, nextDir;
    private int pebbleX, pebbleY;
    private int score, best;
    private int frame;

    SnakeCanvas() {
        setFullScreenMode(true);
    }

    void quit() {
        running = false;
        sound.stop();
    }

    private void layout() {
        int w = getWidth();
        int h = getHeight();
        cols = Math.max(8, w / CELL);
        rows = Math.max(8, (h - HUD) / CELL);
        originX = (w - cols * CELL) / 2;
        originY = HUD + (h - HUD - rows * CELL) / 2;
    }

    private void newGame() {
        layout();
        length = 4;
        for (int i = 0; i < length; i++) {
            xs[i] = cols / 2 - i;
            ys[i] = rows / 2;
        }
        dir = nextDir = RIGHT;
        score = 0;
        placePebble();
        state = PLAY;
        sound.play(0);
    }

    private void placePebble() {
        while (true) {
            int x = (random.nextInt() >>> 1) % cols;
            int y = (random.nextInt() >>> 1) % rows;
            if (!onSnake(x, y, length)) {
                pebbleX = x;
                pebbleY = y;
                return;
            }
        }
    }

    private boolean onSnake(int x, int y, int n) {
        for (int i = 0; i < n; i++) {
            if (xs[i] == x && ys[i] == y) {
                return true;
            }
        }
        return false;
    }

    private void step() {
        if (nextDir != opposite(dir)) {
            dir = nextDir;
        }
        int hx = xs[0], hy = ys[0];
        if (dir == UP) hy--;
        else if (dir == DOWN) hy++;
        else if (dir == LEFT) hx--;
        else hx++;

        boolean eat = hx == pebbleX && hy == pebbleY;
        // The tail cell frees up this step unless we grow, so it is not a collision.
        int body = eat ? length : length - 1;
        if (hx < 0 || hy < 0 || hx >= cols || hy >= rows || onSnake(hx, hy, body)) {
            state = OVER;
            if (score > best) best = score;
            sound.play(3);
            return;
        }
        if (eat && length < MAX) {
            length++;
        }
        for (int i = length - 1; i > 0; i--) {
            xs[i] = xs[i - 1];
            ys[i] = ys[i - 1];
        }
        xs[0] = hx;
        ys[0] = hy;
        if (eat) {
            score += 10;
            sound.play(1);
            placePebble();
        }
    }

    private static int opposite(int d) {
        if (d == UP) return DOWN;
        if (d == DOWN) return UP;
        if (d == LEFT) return RIGHT;
        return LEFT;
    }

    public void run() {
        while (running) {
            if (state == PLAY) {
                step();
            }
            frame++;
            repaint();
            // Starts at 160 ms a step and speeds up with every pebble, down to 70 ms.
            int delay = state == PLAY ? Math.max(70, 160 - score / 2) : 120;
            try {
                Thread.sleep(delay);
            } catch (InterruptedException e) {
                return;
            }
        }
    }

    protected void keyPressed(int key) {
        int action = getGameAction(key);
        // Keypad digits first: handsets differ in which digits they report as game actions.
        if (key == KEY_NUM2) action = UP;
        else if (key == KEY_NUM8) action = DOWN;
        else if (key == KEY_NUM4) action = LEFT;
        else if (key == KEY_NUM6) action = RIGHT;
        else if (key == KEY_NUM5) action = FIRE;

        if (key == KEY_NUM0) {
            sound.enabled = !sound.enabled;
            if (!sound.enabled) sound.stop();
            else sound.play(4);
            repaint();
            return;
        }

        if (state == TITLE || state == OVER) {
            if (action != 0) newGame(); // any direction key or OK starts
        } else if (state == PLAY) {
            if (action == FIRE) {
                state = PAUSED;
                sound.play(4);
            } else if (action == UP || action == DOWN || action == LEFT || action == RIGHT) {
                if (action != opposite(dir) && action != nextDir) {
                    nextDir = action;
                    sound.play(2);
                }
            }
        } else if (state == PAUSED) {
            if (action != 0) {
                state = PLAY;
                sound.play(4);
            }
        }
        repaint();
    }

    protected void paint(Graphics g) {
        int w = getWidth();
        int h = getHeight();
        if (cols == 0) layout();
        g.setColor(WATER_DARK);
        g.fillRect(0, 0, w, h);

        if (state == TITLE) {
            paintTitle(g, w, h);
            return;
        }

        // Field: checkered water.
        for (int y = 0; y < rows; y++) {
            for (int x = 0; x < cols; x++) {
                g.setColor(((x + y) & 1) == 0 ? WATER : WATER_DARK);
                g.fillRect(originX + x * CELL, originY + y * CELL, CELL, CELL);
            }
        }
        // Pebble.
        int px = originX + pebbleX * CELL, py = originY + pebbleY * CELL;
        g.setColor(PEBBLE_SHADE);
        g.fillArc(px + 1, py + 2, CELL - 2, CELL - 3, 0, 360);
        g.setColor(PEBBLE);
        g.fillArc(px + 1, py + 1, CELL - 3, CELL - 4, 0, 360);
        // Otter.
        for (int i = length - 1; i >= 0; i--) {
            g.setColor(i == 0 ? OTTER_HEAD : OTTER);
            g.fillRoundRect(originX + xs[i] * CELL, originY + ys[i] * CELL, CELL, CELL, 4, 4);
        }
        g.setColor(0x000000);
        g.fillRect(originX + xs[0] * CELL + 3, originY + ys[0] * CELL + 3, 2, 2);
        g.fillRect(originX + xs[0] * CELL + CELL - 5, originY + ys[0] * CELL + 3, 2, 2);

        // HUD.
        g.setColor(0x000000);
        g.fillRect(0, 0, w, HUD);
        g.setColor(TEXT);
        g.drawString("점수 " + score, 4, 2, Graphics.TOP | Graphics.LEFT);
        g.drawString((sound.enabled ? "소리 켬" : "소리 끔") + "  최고 " + best, w - 4, 2, Graphics.TOP | Graphics.RIGHT);

        if (state == PAUSED) {
            banner(g, w, h, "일시정지", "아무 방향키나 5 : 계속");
        } else if (state == OVER) {
            banner(g, w, h, "게임 끝 — " + score + "점", "방향키나 5 : 다시 하기");
        }
    }

    private void paintTitle(Graphics g, int w, int h) {
        int cx = w / 2;
        // A little otter swimming across the title.
        int ox = (frame * 3) % (w + 60) - 30;
        for (int i = 4; i >= 0; i--) {
            g.setColor(i == 0 ? OTTER_HEAD : OTTER);
            g.fillRoundRect(ox - i * CELL, h / 5, CELL, CELL, 4, 4);
        }
        g.setColor(PEBBLE);
        g.fillArc(cx - 6, h / 5 + 22, 12, 9, 0, 360);

        g.setColor(ACCENT);
        g.drawString("PEBBLE SNAKE", cx, h / 5 + 40, Graphics.TOP | Graphics.HCENTER);
        g.setColor(TEXT);
        g.drawString("수달이 조약돌을 모아요", cx, h / 5 + 58, Graphics.TOP | Graphics.HCENTER);

        int y = h / 2 + 4;
        int line = Font.getDefaultFont().getHeight() + 4;
        String[] guide = {"방향키 / 2 4 6 8 : 이동", "확인 / 5 : 멈춤 · 계속", "0 : 소리 켜기 · 끄기"};
        for (int i = 0; i < guide.length; i++) {
            g.drawString(guide[i], cx, y + i * line, Graphics.TOP | Graphics.HCENTER);
        }
        if ((frame / 4) % 2 == 0) {
            g.setColor(ACCENT);
            g.drawString("방향키나 5 를 눌러 시작", cx, y + 3 * line + 8, Graphics.TOP | Graphics.HCENTER);
        }
    }

    private static void banner(Graphics g, int w, int h, String title, String hint) {
        int bh = 46;
        int by = (h - bh) / 2;
        g.setColor(0x000000);
        g.fillRect(8, by, w - 16, bh);
        g.setColor(ACCENT);
        g.drawRect(8, by, w - 17, bh - 1);
        g.drawString(title, w / 2, by + 6, Graphics.TOP | Graphics.HCENTER);
        g.setColor(TEXT);
        g.drawString(hint, w / 2, by + 26, Graphics.TOP | Graphics.HCENTER);
    }
}
