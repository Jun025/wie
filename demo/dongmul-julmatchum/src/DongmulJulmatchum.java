/*
 * 동물줄맞춤 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Swap two neighbouring animal faces so three or more of one kind line up; they pop, the
 * faces above drop, and whatever lines up after the drop pops again for a bigger
 * multiplier. The round ends when the time bar runs out. Each animal differs by its
 * silhouette (ears) as well as its colour, so the board still reads without colour.
 * The faces are PNGs drawn at build time by ../art/Faces.java — see why there.
 */

import javax.microedition.lcdui.Graphics;
import javax.microedition.lcdui.Image;

public class DongmulJulmatchum extends Arcade.Midlet {
    Arcade create() {
        return new MatchGame();
    }
}

final class MatchGame extends Arcade {
    private static final int N = 7;
    /** Per level (쉬움, 보통): how many animals, how long a round is. */
    private static final int[] KINDS = {5, 6}, ROUND_MS = {90000, 60000};
    /** Only used if a face image is missing from the jar. */
    private static final int[] COLORS = {0xE2A857, 0x94664C, 0x8B9BB9, 0xEDE8F7, 0x3E4A66, 0x8BC98B};

    private final Image[] small = new Image[6], big = new Image[6];
    private final int[] board = new int[N * N];
    private final boolean[] hit = new boolean[N * N];
    private final boolean[] spark = new boolean[N * N];
    private int kinds = 6;
    private int cx, cy;
    private boolean picked;
    private long sparkUntil;

    MatchGame() {
        for (int k = 0; k < 6; k++) {
            small[k] = load("/f" + k + "_30.png");
            big[k] = load("/f" + k + "_56.png");
        }
    }

    private static Image load(String name) {
        try {
            return Image.createImage(name);
        } catch (Exception e) {
            return null;
        }
    }

    String name() {
        return "동물줄맞춤";
    }

    String tagline() {
        return "같은 동물 셋을 한 줄로";
    }

    int newGame(int level) {
        kinds = KINDS[level];
        fill();
        cx = cy = N / 2;
        picked = false;
        return ROUND_MS[level];
    }

    /** A fresh board with no line already made and at least one move. */
    private void fill() {
        do {
            for (int i = 0; i < N * N; i++) {
                int x = i % N, y = i / N, k;
                do {
                    k = rnd(kinds);
                } while ((x >= 2 && board[i - 1] == k && board[i - 2] == k) || (y >= 2 && board[i - N] == k && board[i - 2 * N] == k));
                board[i] = k;
            }
        } while (!hasMove());
    }

    void key(int action) {
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

    /** Marks every face in a line of three or more in {@code hit}; returns how many. */
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
            // Faces fall into the gaps; new ones drop in from the top.
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
                for (int y = to; y >= 0; y--) board[y * N + x] = rnd(kinds);
            }
        }
        sparkUntil = System.currentTimeMillis() + 350;
        if (chain > 1) {
            toast("연쇄 x" + chain);
            sound.play(Sound.BIG);
            buzz(60);
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

    /** A face centred on (x, y); {@code large} picks the 56px set over the 30px one. */
    private void face(Graphics g, int k, int x, int y, boolean large) {
        if (k < 0) return;
        Image img = (large ? big : small)[k];
        if (img != null) {
            g.drawImage(img, x, y, Graphics.HCENTER | Graphics.VCENTER);
        } else {
            int d = large ? 50 : 26;
            g.setColor(COLORS[k]);
            g.fillArc(x - d / 2, y - d / 2, d, d, 0, 360);
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 16) / N, (h - top - 12) / N);
        int ox = (w - s * N) / 2, oy = top + (h - top - s * N) / 2;
        card(g, ox - 4, oy - 4, s * N + 8, s * N + 8, 18);
        boolean sparkle = System.currentTimeMillis() < sparkUntil;
        for (int i = 0; i < N * N; i++) {
            int x = ox + (i % N) * s, y = oy + (i / N) * s;
            if (sparkle && spark[i]) {
                g.setColor(0xFFF1BF);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, 12, 12);
            } else if (((i % N) + (i / N)) % 2 == 0) {
                g.setColor(0xF7F8FA);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, 12, 12);
            }
            boolean here = state == PLAY && i == cy * N + cx;
            if (here) ring(g, x, y, s, picked ? ACCENT : TEXT, picked ? 3 : 2);
            face(g, board[i], x + s / 2, y + s / 2 - (here && picked ? 2 : 0), false);
        }
    }

    private static void ring(Graphics g, int x, int y, int s, int color, int thick) {
        g.setColor(color);
        for (int t = 0; t < thick; t++) g.drawRoundRect(x + t, y + t, s - 1 - 2 * t, s - 1 - 2 * t, 12, 12);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int[] trio = {0, 1, 3};
        for (int i = 0; i < 3; i++) face(g, trio[i], cx - 62 + i * 62, cy + ((frame / 2 + i) % 3 == 0 ? -3 : 0), true);
    }

    int helpPages() {
        return 4;
    }

    String paintHelp(Graphics g, int page, int top, int w, int bottom) {
        int cx = w / 2;
        int y = top + lineH + 20; // below the page title
        int cap = bottom - 2 * lineH - 12; // two caption lines at the foot of the page
        String[] lines;
        String title;
        if (page == 0) {
            title = "① 바꾸기";
            int[] pic = {0, 0, 2, 3, 4, 0};
            int s = 60, ox = cx - 3 * s / 2;
            card(g, ox - 4, y - 4, 3 * s + 8, 2 * s + 8, 18);
            for (int i = 0; i < 6; i++) face(g, pic[i], ox + (i % 3) * s + s / 2, y + (i / 3) * s + s / 2, true);
            ring(g, ox + 2 * s, y, s, ACCENT, 3);
            ring(g, ox + 2 * s, y + s, s, ACCENT, 1);
            arrow(g, ox + 2 * s + s / 2, y + s - 6);
            lines = new String[] {"5 로 집고 방향키로", "옆 동물과 자리를 바꿔요"};
        } else if (page == 1) {
            title = "② 셋이면 톡";
            int s = 60, ox = cx - 3 * s / 2;
            card(g, ox - 4, y - 4, 3 * s + 8, s + 8, 18);
            g.setColor(0xFFF1BF);
            g.fillRoundRect(ox, y, 3 * s, s, 16, 16);
            for (int i = 0; i < 3; i++) face(g, 0, ox + i * s + s / 2, y + s / 2, true);
            g.setColor(ACCENT);
            bold(g, "+30", cx, y + s + 14, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"같은 동물 셋 이상이 한 줄이면", "사라지고 점수를 얻어요"};
        } else if (page == 2) {
            title = "③ 연쇄";
            int s = 56;
            int[] col = {2, 5, 2};
            for (int i = 0; i < 3; i++) face(g, col[i], cx - 50, y + i * (s - 6) + s / 2, true);
            arrow(g, cx - 50, y + 2);
            g.setColor(TEXT);
            g.fillRoundRect(cx, y + s, 90, lineH + 10, 16, 16);
            g.setColor(WHITE);
            bold(g, "연쇄 x2", cx + 45, y + s + 5, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"빈자리로 떨어져 또 맞으면 연쇄!", "연쇄할수록 점수가 곱절로"};
        } else {
            title = "④ 조작";
            String[][] keys = {{"2 4 6 8", "고르기"}, {"5", "집기 · 놓기"}, {"CLR", "그만하고 처음으로"}};
            for (int i = 0; i < 3; i++) {
                int ky = y + i * (lineH + 14);
                g.setColor(TEXT);
                g.fillRoundRect(28, ky - 4, 70, lineH + 8, 12, 12);
                g.setColor(WHITE);
                bold(g, keys[i][0], 63, ky, Graphics.TOP | Graphics.HCENTER);
                g.setColor(TEXT);
                g.drawString(keys[i][1], 110, ky, Graphics.TOP | Graphics.LEFT);
            }
            int by = y + 3 * (lineH + 14) + 12;
            g.setColor(SURFACE2);
            g.fillRoundRect(28, by, w - 56, 7, 7, 7);
            g.setColor(0xF2B84B);
            g.fillRoundRect(28, by, (w - 56) * 2 / 5, 7, 7, 7);
            lines = new String[] {"위쪽 막대가 시간이에요", "막대가 다 줄면 끝"};
        }
        g.setColor(SUB);
        for (int i = 0; i < lines.length; i++) g.drawString(lines[i], cx, cap + i * (lineH + 4), Graphics.TOP | Graphics.HCENTER);
        return title;
    }

    /** A small down-pointing triangle under (x, y). */
    private static void arrow(Graphics g, int x, int y) {
        g.setColor(ACCENT);
        for (int r = 0; r < 7; r++) g.drawLine(x - 7 + r, y + r, x + 7 - r, y + r);
    }
}
