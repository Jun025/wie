/*
 * 밤송이 찾기 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Ten prickly chestnut burrs hide under an 8x8 patch of leaves. Open a leaf: a number
 * says how many burrs touch it, an empty patch opens its neighbours by itself. Mark the
 * leaves you think hide a burr, open every other leaf to finish. The first leaf you open
 * is never a burr (they are placed after it, away from it). Faster finishes score more.
 */

import javax.microedition.lcdui.Graphics;

public class BamsongiField extends Arcade.Midlet {
    Arcade create() {
        return new BurrGame();
    }
}

final class BurrGame extends Arcade {
    private static final int N = 8, BURRS = 10;
    private static final int LEAF = 0xB9DDC4, LEAF_DARK = 0x8CC3A1, BURR = 0x9A6B45, FLAG = 0xE5342F;
    private static final int[] DIGIT = {TEXT, 0x2F6FE5, 0x0D7A54, 0x9A5C00, 0xD11F1A, 0xD11F1A, 0xD11F1A, 0xD11F1A, 0xD11F1A};

    private final boolean[] burr = new boolean[N * N];
    private final boolean[] open = new boolean[N * N];
    private final boolean[] flag = new boolean[N * N];
    private final int[] stack = new int[N * N];
    private int cx, cy, flags, opened, boom = -1;
    private boolean placed;
    private long startAt, doneMs;

    String name() {
        return "밤송이 찾기";
    }

    String tagline() {
        return "숨은 밤송이 10개를 피해요";
    }

    String[] guide() {
        return new String[] {"방향키 / 2 4 6 8 : 고르기", "5 : 열기", "# 또는 * : 밤송이 표시"};
    }

    void newGame() {
        for (int i = 0; i < N * N; i++) burr[i] = open[i] = flag[i] = false;
        cx = cy = N / 2;
        flags = opened = 0;
        boom = -1;
        placed = false;
        startAt = System.currentTimeMillis();
    }

    private int secs() {
        return (int) ((state == PLAY ? System.currentTimeMillis() - startAt : doneMs) / 1000);
    }

    String status() {
        return "남은 " + (BURRS - flags) + " · " + secs() + "초";
    }

    int delay() {
        return 250;
    }

    void key(int action, int key) {
        int i = cy * N + cx;
        if (key == KEY_POUND || key == KEY_STAR) {
            if (!open[i]) {
                flag[i] = !flag[i];
                flags += flag[i] ? 1 : -1;
                sound.play(Sound.MOVE);
            }
            return;
        }
        if (action == FIRE) {
            if (open[i] || flag[i]) return;
            if (!placed) place(i);
            if (burr[i]) {
                boom = i;
                doneMs = System.currentTimeMillis() - startAt;
                gameOver(false);
                return;
            }
            reveal(i);
            if (opened == N * N - BURRS) {
                doneMs = System.currentTimeMillis() - startAt;
                score += Math.max(50, 400 - secs() * 2);
                gameOver(true);
            } else {
                sound.play(Sound.GOOD);
            }
            return;
        }
        if (action == LEFT && cx > 0) cx--;
        else if (action == RIGHT && cx < N - 1) cx++;
        else if (action == UP && cy > 0) cy--;
        else if (action == DOWN && cy < N - 1) cy++;
    }

    /** Burrs go anywhere except the first leaf and its eight neighbours. */
    private void place(int first) {
        int left = BURRS;
        while (left > 0) {
            int i = rnd(N * N);
            if (burr[i] || Math.abs(i % N - first % N) <= 1 && Math.abs(i / N - first / N) <= 1) continue;
            burr[i] = true;
            left--;
        }
        placed = true;
    }

    private int around(int i) {
        int n = 0;
        for (int dy = -1; dy <= 1; dy++) {
            for (int dx = -1; dx <= 1; dx++) {
                int x = i % N + dx, y = i / N + dy;
                if (x >= 0 && y >= 0 && x < N && y < N && burr[y * N + x]) n++;
            }
        }
        return n;
    }

    /** Opens a leaf; an empty one opens its neighbours too (an explicit stack, no recursion). */
    private void reveal(int start) {
        int sp = 0;
        stack[sp++] = start;
        while (sp > 0) {
            int i = stack[--sp];
            if (open[i] || flag[i]) continue;
            open[i] = true;
            opened++;
            score += 5;
            if (around(i) != 0) continue;
            for (int dy = -1; dy <= 1; dy++) {
                for (int dx = -1; dx <= 1; dx++) {
                    int x = i % N + dx, y = i / N + dy;
                    if (x >= 0 && y >= 0 && x < N && y < N && !open[y * N + x] && sp < stack.length) stack[sp++] = y * N + x;
                }
            }
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 20) / N, (h - top - 24) / N);
        int ox = (w - s * N) / 2, oy = top + (h - top - s * N) / 2;
        g.setColor(WHITE);
        g.fillRoundRect(ox - 6, oy - 6, s * N + 12, s * N + 12, 18, 18);
        for (int i = 0; i < N * N; i++) {
            int x = ox + (i % N) * s + 2, y = oy + (i / N) * s + 2, d = s - 4;
            boolean here = state == PLAY && i == cy * N + cx;
            boolean show = open[i] || (state == OVER && burr[i]);
            tile(g, x, y, d, show ? (i == boom ? 0xFDECEC : SURFACE) : ((i % N + i / N) % 2 == 0 ? LEAF : LEAF_DARK), here);
            if (show && burr[i]) {
                burrAt(g, x + d / 2, y + d / 2, d / 2 - 2);
            } else if (open[i]) {
                int n = around(i);
                if (n > 0) {
                    g.setColor(DIGIT[n]);
                    bold(g, "" + n, x + d / 2, y + (d - lineH) / 2, Graphics.TOP | Graphics.HCENTER);
                }
            } else if (flag[i]) {
                g.setColor(TEXT);
                g.fillRect(x + d / 2 - 3, y + 4, 2, d - 8);
                g.setColor(FLAG);
                g.fillRoundRect(x + d / 2 - 1, y + 4, d / 3 + 1, d / 3, 4, 4);
            }
        }
    }

    private static void burrAt(Graphics g, int cx, int cy, int r) {
        g.setColor(BURR);
        for (int a = 0; a < 8; a++) {
            int dx = a == 0 || a == 4 ? 0 : a < 4 ? 1 : -1;
            int dy = a == 2 || a == 6 ? 0 : a < 2 || a > 6 ? -1 : 1;
            g.drawLine(cx, cy, cx + dx * (r + 2), cy + dy * (r + 2));
        }
        g.fillArc(cx - r + 2, cy - r + 2, 2 * r - 4, 2 * r - 4, 0, 360);
        g.setColor(0xD9B48F);
        g.fillArc(cx - r / 3, cy - r / 3, r / 2 + 1, r / 2 + 1, 0, 360);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        tile(g, cx - 46, cy - 14, 28, LEAF, false);
        tile(g, cx - 14, cy - 14, 28, SURFACE2, false);
        g.setColor(DIGIT[2]);
        bold(g, "2", cx, cy - 6, Graphics.TOP | Graphics.HCENTER);
        tile(g, cx + 18, cy - 14, 28, SURFACE2, false);
        burrAt(g, cx + 32, cy, 12 + (frame / 3) % 2);
    }
}
