/*
 * 차곡차곡 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Small pieces of one to three cells fall into an 8x13 box. Two things clear cells:
 * a row filled edge to edge, and — the rule this game is built around — three or more
 * cells of one colour touching in a straight line, across or down. After a clear every
 * cell drops straight down on its own, so one landing can set off a chain; each link
 * of the chain multiplies the points. The round ends when a new piece has no room.
 *
 * Pieces: a single cell, a pair, a straight three, a corner three, and a round
 * "pop" cell that clears the 3x3 around where it lands. Colour is per cell, drawn from
 * one four-colour set, never tied to a piece's shape. One cell per step, keys act on
 * the next frame, so the game stays fair when the engine runs at 4 fps.
 */

import javax.microedition.lcdui.Graphics;

public class ChagokStack extends Arcade.Midlet {
    Arcade create() {
        return new StackGame();
    }
}

final class StackGame extends Arcade {
    private static final int W = 8, H = 13, POP = 9;
    private static final int[] COLORS = {0, 0xF39A8B, 0x7FCBAE, 0x86AEEA, 0xF2C766};
    // Shapes as {dx0, dy0, dx1, dy1, …}: single, pair, straight three, corner three.
    private static final int[][] SHAPES = {{0, 0}, {0, 0, 1, 0}, {0, 0, 1, 0, 2, 0}, {0, 0, 1, 0, 0, 1}};

    private final int[] box = new int[W * H];
    private final boolean[] gone = new boolean[W * H];
    private final boolean[] spark = new boolean[W * H];
    private int[] px = new int[3], py = new int[3], pc = new int[3];
    private int cells, x, y, pieces, bestChain;
    private long nextFall, sparkUntil;

    String name() {
        return "차곡차곡";
    }

    String tagline() {
        return "줄을 채우거나 같은 색 셋";
    }

    String[] guide() {
        return new String[] {"4 6 / 좌우 방향키 : 옮기기", "2 / 5 / 위 방향키 : 돌리기", "8 / 아래 방향키 : 바로 내리기", "같은 색 3칸 한 줄도 사라져요"};
    }

    void newGame() {
        for (int i = 0; i < W * H; i++) box[i] = 0;
        pieces = bestChain = 0;
        spawn();
    }

    String status() {
        return "단계 " + level();
    }

    private int level() {
        return pieces / 12 + 1;
    }

    int delay() {
        return 100;
    }

    private int fallMs() {
        return Math.max(300, 800 - (level() - 1) * 60);
    }

    private void spawn() {
        int r = rnd(20);
        if (r == 0) {
            cells = 1;
            px[0] = py[0] = 0;
            pc[0] = POP;
        } else {
            int[] s = SHAPES[r < 5 ? 0 : r < 11 ? 1 : r < 15 ? 2 : 3];
            cells = s.length / 2;
            for (int i = 0; i < cells; i++) {
                px[i] = s[2 * i];
                py[i] = s[2 * i + 1];
                pc[i] = 1 + rnd(4);
            }
        }
        x = (W - width()) / 2;
        y = 0;
        pieces++;
        nextFall = System.currentTimeMillis() + fallMs();
        if (!fits(x, y, px, py)) gameOver(false);
    }

    private int width() {
        int m = 0;
        for (int i = 0; i < cells; i++) m = Math.max(m, px[i] + 1);
        return m;
    }

    private boolean fits(int ax, int ay, int[] dx, int[] dy) {
        for (int i = 0; i < cells; i++) {
            int cx = ax + dx[i], cy = ay + dy[i];
            if (cx < 0 || cx >= W || cy >= H || cy >= 0 && box[cy * W + cx] != 0) return false;
        }
        return true;
    }

    void tick(long now) {
        if (now < nextFall) return;
        nextFall = now + fallMs();
        if (fits(x, y + 1, px, py)) y++;
        else land();
    }

    void key(int action, int key) {
        if (action == LEFT && fits(x - 1, y, px, py)) x--;
        else if (action == RIGHT && fits(x + 1, y, px, py)) x++;
        else if (action == UP || action == FIRE) rotate();
        else if (action == DOWN) {
            while (fits(x, y + 1, px, py)) y++;
            land();
        }
    }

    /** A quarter turn; if it hits a wall, try one cell left or right before giving up. */
    private void rotate() {
        if (cells == 1) return;
        int[] nx = new int[3], ny = new int[3];
        int minX = 9, minY = 9;
        for (int i = 0; i < cells; i++) {
            nx[i] = -py[i];
            ny[i] = px[i];
            minX = Math.min(minX, nx[i]);
            minY = Math.min(minY, ny[i]);
        }
        for (int i = 0; i < cells; i++) {
            nx[i] -= minX;
            ny[i] -= minY;
        }
        for (int kick = 0; kick < 3; kick++) {
            int ax = x + (kick == 1 ? -1 : kick == 2 ? 1 : 0);
            if (fits(ax, y, nx, ny)) {
                x = ax;
                px = nx;
                py = ny;
                return;
            }
        }
    }

    private void land() {
        for (int i = 0; i < W * H; i++) spark[i] = false;
        if (pc[0] == POP) {
            int n = 0;
            for (int dy = -1; dy <= 1; dy++) {
                for (int dx = -1; dx <= 1; dx++) {
                    int cx = x + dx, cy = y + dy;
                    if (cx >= 0 && cy >= 0 && cx < W && cy < H) {
                        if (box[cy * W + cx] != 0) n++;
                        box[cy * W + cx] = 0;
                        spark[cy * W + cx] = true;
                    }
                }
            }
            score += n * 10;
            settle();
        } else {
            for (int i = 0; i < cells; i++) {
                int cy = y + py[i];
                if (cy >= 0) box[cy * W + x + px[i]] = pc[i];
            }
        }
        int chain = 0;
        int n;
        while ((n = mark()) > 0) {
            chain++;
            score += n * 10 * chain;
            for (int i = 0; i < W * H; i++) {
                if (gone[i]) {
                    box[i] = 0;
                    spark[i] = true;
                }
            }
            settle();
        }
        sparkUntil = System.currentTimeMillis() + 400;
        if (chain > 1) {
            toast("연쇄 x" + chain);
            sound.play(Sound.BIG);
        } else if (chain == 1 || pc[0] == POP) {
            sound.play(Sound.GOOD);
        } else {
            sound.play(Sound.MOVE);
        }
        bestChain = Math.max(bestChain, chain);
        spawn();
    }

    /** Marks full rows and same-colour straight runs of three or more; returns the count. */
    private int mark() {
        for (int i = 0; i < W * H; i++) gone[i] = false;
        for (int r = 0; r < H; r++) {
            boolean full = true;
            for (int c = 0; c < W; c++) full &= box[r * W + c] != 0;
            if (full) for (int c = 0; c < W; c++) gone[r * W + c] = true;
        }
        for (int r = 0; r < H; r++) {
            for (int c = 0; c < W; c++) {
                int k = box[r * W + c];
                if (k == 0) continue;
                int run = 1;
                while (c + run < W && box[r * W + c + run] == k) run++;
                if (run >= 3) for (int i = 0; i < run; i++) gone[r * W + c + i] = true;
                run = 1;
                while (r + run < H && box[(r + run) * W + c] == k) run++;
                if (run >= 3) for (int i = 0; i < run; i++) gone[(r + i) * W + c] = true;
            }
        }
        int n = 0;
        for (int i = 0; i < W * H; i++) if (gone[i]) n++;
        return n;
    }

    /** Every cell drops straight down until it rests on something. */
    private void settle() {
        for (int c = 0; c < W; c++) {
            int to = H - 1;
            for (int r = H - 1; r >= 0; r--) {
                int k = box[r * W + c];
                if (k != 0) {
                    box[r * W + c] = 0;
                    box[to * W + c] = k;
                    to--;
                }
            }
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((h - top - 12) / H, (w - 76) / W);
        int ox = 10, oy = top + (h - top - s * H) / 2;
        g.setColor(WHITE);
        g.fillRoundRect(ox - 4, oy - 4, s * W + 8, s * H + 8, 14, 14);
        boolean sparkle = System.currentTimeMillis() < sparkUntil;
        for (int i = 0; i < W * H; i++) {
            int cx = ox + (i % W) * s, cy = oy + (i / W) * s;
            if (sparkle && spark[i]) {
                g.setColor(0xFFF3C4);
                g.fillRoundRect(cx, cy, s, s, 8, 8);
            } else if (box[i] == 0) {
                g.setColor(SURFACE);
                g.fillRect(cx + s / 2 - 1, cy + s / 2 - 1, 2, 2);
            }
            if (box[i] != 0) cell(g, box[i], cx + 1, cy + 1, s - 2);
        }
        if (state == PLAY) {
            for (int i = 0; i < cells; i++) {
                if (y + py[i] >= 0) cell(g, pc[i], ox + (x + px[i]) * s + 1, oy + (y + py[i]) * s + 1, s - 2);
            }
        }
        int sx = ox + s * W + 12, sw = w - sx - 6;
        g.setColor(WHITE);
        g.fillRoundRect(sx, oy - 4, sw, 3 * lineH + 30, 12, 12);
        g.setColor(SUB);
        g.drawString("조각", sx + sw / 2, oy + 2, Graphics.TOP | Graphics.HCENTER);
        g.setColor(TEXT);
        bold(g, "" + pieces, sx + sw / 2, oy + lineH + 4, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("연쇄 " + bestChain, sx + sw / 2, oy + 2 * lineH + 12, Graphics.TOP | Graphics.HCENTER);
    }

    private static void cell(Graphics g, int k, int x, int y, int d) {
        if (k == POP) {
            g.setColor(TEXT);
            g.fillArc(x, y, d, d, 0, 360);
            g.setColor(WHITE);
            g.fillArc(x + d / 4, y + d / 4, d / 4 + 1, d / 4 + 1, 0, 360);
            return;
        }
        g.setColor(COLORS[k]);
        g.fillRoundRect(x, y, d, d, d / 2, d / 2);
        // A shape mark per colour, so colour is not the only cue.
        g.setColor(WHITE);
        int m = d / 2;
        if (k == 1) g.fillArc(x + m - 2, y + m - 2, 5, 5, 0, 360);
        else if (k == 2) g.fillRect(x + m - 4, y + m - 1, 8, 2);
        else if (k == 3) g.fillRect(x + m - 1, y + m - 4, 2, 8);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int drop = (frame % 6) * 3;
        cell(g, 2, cx - 10, cy - 24 + drop, 18);
        cell(g, 2, cx - 30, cy + 6, 18);
        cell(g, 2, cx + 10, cy + 6, 18);
        cell(g, 1, cx - 10, cy + 6, 18);
        cell(g, 3, cx + 30, cy + 6, 18);
    }
}
