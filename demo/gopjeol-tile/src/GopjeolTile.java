/*
 * 곱절 타일 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Push every tile one way; two tiles of the same step that meet become one tile a step
 * higher. A new step-1 tile appears after each push. Reach step 8 to finish the round;
 * the round ends early when no push can move anything. Tiles show their step (1~8),
 * and the colour deepens every two steps.
 */

import javax.microedition.lcdui.Graphics;

public class GopjeolTile extends Arcade.Midlet {
    Arcade create() {
        return new MergeGame();
    }
}

final class MergeGame extends Arcade {
    private static final int N = 4, GOAL = 8;
    private static final int[] COLORS = {0xFFFFFF, 0xE3E7EC, 0xD3EDE1, 0xA6DAC2, 0xD6E4F8, 0x9CC0F0, 0xF8E1B4, 0xF2BE6E, 0xE89A7C};

    private final int[] board = new int[N * N];
    private final boolean[] merged = new boolean[N * N];
    private int top, born = -1;
    private long flashUntil;

    String name() {
        return "곱절 타일";
    }

    String tagline() {
        return "같은 숫자를 밀어 합쳐요";
    }

    String[] guide() {
        return new String[] {"방향키 / 2 4 6 8 : 밀기", "같은 숫자가 만나면 +1", "8 을 만들면 완성"};
    }

    void newGame() {
        for (int i = 0; i < N * N; i++) board[i] = 0;
        top = 1;
        spawn();
        spawn();
    }

    String status() {
        return "목표 " + top + " / " + GOAL;
    }

    private void spawn() {
        int free = 0;
        for (int i = 0; i < N * N; i++) if (board[i] == 0) free++;
        if (free == 0) return;
        int pick = rnd(free);
        for (int i = 0; i < N * N; i++) {
            if (board[i] == 0 && pick-- == 0) {
                board[i] = rnd(10) == 0 ? 2 : 1;
                born = i;
                return;
            }
        }
    }

    void key(int action, int key) {
        int dx = action == LEFT ? -1 : action == RIGHT ? 1 : 0;
        int dy = action == UP ? -1 : action == DOWN ? 1 : 0;
        if (dx == 0 && dy == 0) return;
        if (!push(dx, dy, true)) return;
        spawn();
        flashUntil = System.currentTimeMillis() + 300;
        if (top >= GOAL) {
            score += 500;
            gameOver(true);
        } else if (!push(1, 0, false) && !push(-1, 0, false) && !push(0, 1, false) && !push(0, -1, false)) {
            gameOver(false);
        }
    }

    /** Pushes toward (dx, dy). With {@code apply} false it only answers "would anything move?". */
    private boolean push(int dx, int dy, boolean apply) {
        boolean moved = false;
        boolean anyMerge = false;
        if (apply) for (int i = 0; i < N * N; i++) merged[i] = false;
        for (int line = 0; line < N; line++) {
            // Walk the line starting from the edge the tiles move toward.
            int[] idx = new int[N];
            for (int k = 0; k < N; k++) {
                int p = dx + dy > 0 ? N - 1 - k : k;
                idx[k] = dx != 0 ? line * N + p : p * N + line;
            }
            int[] out = new int[N];
            boolean[] joined = new boolean[N];
            int n = 0;
            for (int k = 0; k < N; k++) {
                int v = board[idx[k]];
                if (v == 0) continue;
                if (n > 0 && out[n - 1] == v && !joined[n - 1]) {
                    out[n - 1] = v + 1;
                    joined[n - 1] = true;
                } else {
                    out[n++] = v;
                }
            }
            for (int k = 0; k < N; k++) {
                if (board[idx[k]] != out[k]) moved = true;
                if (apply) {
                    board[idx[k]] = out[k];
                    merged[idx[k]] = joined[k];
                    if (joined[k]) {
                        anyMerge = true;
                        score += out[k] * 10;
                        if (out[k] > top) top = out[k];
                    }
                }
            }
        }
        if (apply && moved) sound.play(anyMerge ? Sound.GOOD : Sound.MOVE);
        return moved;
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 24) / N, (h - top - 30) / N);
        int ox = (w - s * N) / 2, oy = top + (h - top - s * N) / 2;
        g.setColor(SURFACE2);
        g.fillRoundRect(ox - 6, oy - 6, s * N + 12, s * N + 12, 20, 20);
        boolean flash = System.currentTimeMillis() < flashUntil;
        for (int i = 0; i < N * N; i++) {
            int x = ox + (i % N) * s + 3, y = oy + (i / N) * s + 3, d = s - 6;
            int v = board[i];
            boolean pop = flash && (merged[i] || i == born);
            tile(g, x, y, d, COLORS[Math.min(v, COLORS.length - 1)], pop && merged[i]);
            if (v == 0) continue;
            if (pop && i == born && !merged[i]) {
                g.setColor(WHITE);
                g.fillRoundRect(x + 4, y + 4, d - 8, d - 8, 10, 10);
                g.setColor(COLORS[v]);
                g.fillRoundRect(x + 7, y + 7, d - 14, d - 14, 8, 8);
            }
            g.setColor(TEXT);
            bold(g, "" + v, x + d / 2, y + d / 2 - lineH / 2, Graphics.TOP | Graphics.HCENTER);
            // Pips under the number repeat the step, so two tiles can be told apart without colour.
            for (int p = 0; p < v; p++) g.fillRect(x + d / 2 - v * 3 + p * 6 + 1, y + d - 9, 4, 3);
        }
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int shift = (frame / 3) % 4 == 0 ? 6 : 0;
        tile(g, cx - 44 + shift, cy - 16, 30, COLORS[3], false);
        tile(g, cx - 10 - shift, cy - 16, 30, COLORS[3], false);
        tile(g, cx + 30, cy - 16, 30, COLORS[4], true);
        g.setColor(TEXT);
        bold(g, "3", cx - 29 + shift, cy - 7, Graphics.TOP | Graphics.HCENTER);
        bold(g, "3", cx + 5 - shift, cy - 7, Graphics.TOP | Graphics.HCENTER);
        bold(g, "4", cx + 45, cy - 7, Graphics.TOP | Graphics.HCENTER);
    }
}
