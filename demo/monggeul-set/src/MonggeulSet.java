/*
 * 몽글셋 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Swap two neighbouring beads so three or more of one kind line up; they pop, the
 * beads above drop, and whatever lines up after the drop pops again for a bigger
 * multiplier. One round is 60 seconds. Each kind has its own shape as well as its
 * own colour, so the board still reads without colour.
 */

import javax.microedition.lcdui.Graphics;

public class MonggeulSet extends Arcade.Midlet {
    Arcade create() {
        return new MatchGame();
    }
}

final class MatchGame extends Arcade {
    private static final int N = 7, KINDS = 5, ROUND_MS = 60000;
    private static final int[] COLORS = {0xF4A08C, 0xF2C766, 0x86CFAE, 0x8DB5EE, 0xB8A1E6};

    private final int[] board = new int[N * N];
    private final boolean[] hit = new boolean[N * N];
    private final boolean[] spark = new boolean[N * N];
    private int cx, cy;
    private boolean picked;
    private long endAt, sparkUntil;

    String name() {
        return "몽글셋";
    }

    String tagline() {
        return "같은 알 셋을 한 줄로";
    }

    String[] guide() {
        return new String[] {"방향키 / 2 4 6 8 : 고르기", "5 로 집고 방향키로 바꾸기", "60초 동안 많이 터뜨리기"};
    }

    void newGame() {
        fill();
        cx = cy = N / 2;
        picked = false;
        endAt = System.currentTimeMillis() + ROUND_MS;
    }

    String status() {
        long left = state == PLAY ? Math.max(0, endAt - System.currentTimeMillis()) : 0;
        return (left + 999) / 1000 + "초";
    }

    void tick(long now) {
        if (now >= endAt) gameOver(false);
    }

    /** A fresh board with no line already made and at least one move. */
    private void fill() {
        do {
            for (int i = 0; i < N * N; i++) {
                int x = i % N, y = i / N, k;
                do {
                    k = rnd(KINDS);
                } while ((x >= 2 && board[i - 1] == k && board[i - 2] == k) || (y >= 2 && board[i - N] == k && board[i - 2 * N] == k));
                board[i] = k;
            }
        } while (!hasMove());
    }

    void key(int action, int key) {
        int dx = action == LEFT ? -1 : action == RIGHT ? 1 : 0;
        int dy = action == UP ? -1 : action == DOWN ? 1 : 0;
        if (action == FIRE) {
            picked = !picked;
            return;
        }
        if (dx == 0 && dy == 0) return;
        int nx = cx + dx, ny = cy + dy;
        if (nx < 0 || ny < 0 || nx >= N || ny >= N) return;
        if (!picked) {
            cx = nx;
            cy = ny;
            return;
        }
        picked = false;
        swap(cy * N + cx, ny * N + nx);
        if (findLines() == 0) {
            swap(cy * N + cx, ny * N + nx);
            sound.play(Sound.BAD);
            return;
        }
        cx = nx;
        cy = ny;
        resolve();
    }

    private void swap(int a, int b) {
        int t = board[a];
        board[a] = board[b];
        board[b] = t;
    }

    /** Marks every bead in a line of three or more in {@code hit}; returns how many. */
    private int findLines() {
        int count = 0;
        for (int i = 0; i < N * N; i++) hit[i] = false;
        for (int y = 0; y < N; y++) {
            for (int x = 0; x < N; x++) {
                int k = board[y * N + x];
                if (k < 0) continue;
                int run = 1;
                while (x + run < N && board[y * N + x + run] == k) run++;
                if (run >= 3) for (int r = 0; r < run; r++) hit[y * N + x + r] = true;
                run = 1;
                while (y + run < N && board[(y + run) * N + x] == k) run++;
                if (run >= 3) for (int r = 0; r < run; r++) hit[(y + r) * N + x] = true;
            }
        }
        for (int i = 0; i < N * N; i++) if (hit[i]) count++;
        return count;
    }

    private void resolve() {
        for (int i = 0; i < N * N; i++) spark[i] = false;
        int chain = 0;
        int popped;
        while ((popped = findLines()) > 0) {
            chain++;
            score += popped * 10 * chain;
            for (int i = 0; i < N * N; i++) {
                if (hit[i]) {
                    board[i] = -1;
                    spark[i] = true;
                }
            }
            // Beads fall into the gaps; new ones drop in from the top.
            for (int x = 0; x < N; x++) {
                int to = N - 1;
                for (int y = N - 1; y >= 0; y--) {
                    int k = board[y * N + x];
                    if (k >= 0) {
                        board[y * N + x] = -1;
                        board[to * N + x] = k;
                        to--;
                    }
                }
                for (int y = to; y >= 0; y--) board[y * N + x] = rnd(KINDS);
            }
        }
        sparkUntil = System.currentTimeMillis() + 350;
        if (chain > 1) {
            toast("연쇄 x" + chain);
            sound.play(Sound.BIG);
        } else {
            sound.play(Sound.GOOD);
        }
        if (!hasMove()) {
            fill();
            toast("판을 새로 섞었어요");
        }
    }

    private boolean hasMove() {
        for (int i = 0; i < N * N; i++) {
            int x = i % N;
            if (x + 1 < N && swapMakesLine(i, i + 1)) return true;
            if (i + N < N * N && swapMakesLine(i, i + N)) return true;
        }
        return false;
    }

    private boolean swapMakesLine(int a, int b) {
        swap(a, b);
        boolean ok = findLines() > 0;
        swap(a, b);
        return ok;
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 20) / N, (h - top - 24) / N);
        int ox = (w - s * N) / 2, oy = top + (h - top - s * N) / 2;
        g.setColor(WHITE);
        g.fillRoundRect(ox - 6, oy - 6, s * N + 12, s * N + 12, 18, 18);
        boolean sparkle = System.currentTimeMillis() < sparkUntil;
        for (int i = 0; i < N * N; i++) {
            int x = ox + (i % N) * s, y = oy + (i / N) * s;
            boolean here = state == PLAY && i == cy * N + cx;
            if (here) {
                g.setColor(TEXT);
                g.fillRoundRect(x, y, s, s, 12, 12);
                g.setColor(picked ? SURFACE2 : WHITE);
                g.fillRoundRect(x + 3, y + 3, s - 6, s - 6, 9, 9);
            }
            if (sparkle && spark[i]) {
                g.setColor(0xFFF3C4);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, 10, 10);
            }
            bead(g, board[i], x + 4, y + (here && picked ? 2 : 4), s - 8);
        }
    }

    private static void bead(Graphics g, int k, int x, int y, int d) {
        if (k < 0) return;
        g.setColor(COLORS[k]);
        int q = d / 4;
        switch (k) {
            case 0: // round
                g.fillArc(x, y, d, d, 0, 360);
                break;
            case 1: // soft square
                g.fillRoundRect(x + 1, y + 1, d - 2, d - 2, d / 2, d / 2);
                break;
            case 2: // pill
                g.fillRoundRect(x, y + q, d, d - 2 * q, d / 2, d / 2);
                break;
            case 3: // ring
                g.fillArc(x, y, d, d, 0, 360);
                g.setColor(WHITE);
                g.fillArc(x + q + 1, y + q + 1, d - 2 * q - 2, d - 2 * q - 2, 0, 360);
                break;
            default: // square with a dot
                g.fillRoundRect(x + 1, y + 1, d - 2, d - 2, 6, 6);
                g.setColor(WHITE);
                g.fillArc(x + d / 2 - 3, y + d / 2 - 3, 6, 6, 0, 360);
        }
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        for (int k = 0; k < 3; k++) bead(g, k * 2, cx - 40 + k * 28, cy - 12 + ((frame / 2 + k) % 3 == 0 ? -3 : 0), 24);
    }
}
