/*
 * 주렁주렁 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Two fruits fall together into a 7x11 crate. Steer and turn the pair, drop it, and when
 * three or more of one fruit touch they pop. The rule this game is built around: fruit
 * hangs in bunches, so a DIAGONAL touch joins a bunch too — three strawberries in a slant
 * are a bunch. After a pop everything above falls, and if that makes a new bunch it pops
 * as well: a chain. Each link of the chain multiplies the points, and a chain of n earns
 * n "easy" pairs that fall at half speed. The round ends when the top middle is full.
 *
 * Five fruits, each with its own outline mark (seeds, leaf, grapes, stem, crown), so the
 * colour is never the only cue. Everything is counted in frames, never in milliseconds:
 * a pair falls one cell every few frames, and a chain moves one step per frame — one row
 * of settling, or one frame of popping — so at 4 fps the chain is slower, not skipped.
 */

import javax.microedition.lcdui.Graphics;

public class JureongBerry extends Arcade.Midlet {
    Arcade create() {
        return new BerryGame();
    }
}

final class BerryGame extends Arcade {
    private static final int W = 7, H = 11, KINDS = 5, BUNCH = 3, SPAWN = 3;
    private static final int FALL = 0, SETTLE = 1, POP = 2;
    // strawberry, tangerine, grape, green apple, blueberry
    private static final int[] COLORS = {0, 0xF0848B, 0xF5AE5E, 0xAE92DC, 0x9DCF72, 0x7FA8EA};
    private static final int LEAF = 0x6DB36F, CRATE = 0xF6ECDC, SLAT = 0xEADBC3;
    // Where the second fruit sits around the first: above, right, below, left.
    private static final int[] DX = {0, 1, 0, -1}, DY = {-1, 0, 1, 0};

    private final int[] box = new int[W * H];
    private final boolean[] pop = new boolean[W * H];
    private int phase, x, y, dir, a, b, nextA, nextB, fallCount, popFrames;
    private int pairs, chain, bestChain, easy;

    String name() {
        return "주렁주렁";
    }

    String tagline() {
        return "같은 열매 셋이 한 송이면 톡";
    }

    String[] guide() {
        return new String[] {"4 6 / 좌우 방향키 : 옮기기", "2 / 위 방향키 : 돌리기", "5 8 / 아래 방향키 : 바로 내리기", "대각선으로 닿아도 한 송이예요"};
    }

    void newGame() {
        for (int i = 0; i < W * H; i++) {
            box[i] = 0;
            pop[i] = false;
        }
        pairs = chain = bestChain = easy = 0;
        nextA = 1 + rnd(KINDS);
        nextB = 1 + rnd(KINDS);
        spawn();
    }

    String status() {
        return "단계 " + level();
    }

    private int level() {
        return pairs / 15 + 1;
    }

    int delay() {
        return 120;
    }

    /** Frames per cell of falling: 6 at stage 1, down to 2; an easy pair takes twice as long. */
    private int fallEvery() {
        int f = Math.max(2, 7 - level());
        return easy > 0 ? 2 * f : f;
    }

    private void spawn() {
        a = nextA;
        b = nextB;
        nextA = 1 + rnd(KINDS);
        nextB = 1 + rnd(KINDS);
        x = SPAWN;
        y = 1;
        dir = 0;
        fallCount = 0;
        pairs++;
        phase = FALL;
        if (!fits(x, y, dir)) gameOver(false);
    }

    private boolean free(int cx, int cy) {
        return cx >= 0 && cx < W && cy >= 0 && cy < H && box[cy * W + cx] == 0;
    }

    private boolean fits(int px, int py, int d) {
        return free(px, py) && free(px + DX[d], py + DY[d]);
    }

    void tick(long now) {
        if (phase == FALL) {
            if (++fallCount < fallEvery()) return;
            fallCount = 0;
            if (fits(x, y + 1, dir)) y++;
            else place();
        } else if (phase == POP) {
            if (--popFrames > 0) return;
            for (int i = 0; i < W * H; i++) {
                if (pop[i]) {
                    box[i] = 0;
                    pop[i] = false;
                }
            }
            phase = SETTLE;
        } else if (!settleStep()) {
            int n = mark();
            if (n > 0) {
                chain++;
                score += n * 10 * chain;
                popFrames = 2;
                phase = POP;
                sound.play(chain > 1 ? Sound.BIG : Sound.GOOD);
            } else {
                if (chain > 1) easy += chain;
                bestChain = Math.max(bestChain, chain);
                spawn();
            }
        }
    }

    void key(int action, int key) {
        if (phase != FALL) return;
        if (action == LEFT && fits(x - 1, y, dir)) x--;
        else if (action == RIGHT && fits(x + 1, y, dir)) x++;
        else if (action == UP) turn();
        else if (action == FIRE || action == DOWN) {
            while (fits(x, y + 1, dir)) y++;
            place();
        }
    }

    /** A quarter turn; against a wall or a fruit, the pair steps one cell away and tries again. */
    private void turn() {
        int d = (dir + 1) % 4;
        if (fits(x, y, d)) dir = d;
        else if (fits(x - DX[d], y - DY[d], d)) {
            x -= DX[d];
            y -= DY[d];
            dir = d;
        }
    }

    private void place() {
        box[y * W + x] = a;
        box[(y + DY[dir]) * W + x + DX[dir]] = b;
        if (easy > 0) easy--;
        chain = 0;
        phase = SETTLE;
        sound.play(Sound.MOVE);
    }

    /** Every fruit with a gap below it drops one row. Returns whether anything moved. */
    private boolean settleStep() {
        boolean moved = false;
        for (int r = H - 2; r >= 0; r--) {
            for (int c = 0; c < W; c++) {
                int i = r * W + c;
                if (box[i] != 0 && box[i + W] == 0) {
                    box[i + W] = box[i];
                    box[i] = 0;
                    moved = true;
                }
            }
        }
        return moved;
    }

    /** Marks every bunch of BUNCH or more — eight neighbours join — and returns the count. */
    private int mark() {
        int n = 0;
        boolean[] seen = new boolean[W * H];
        int[] stack = new int[W * H];
        int[] group = new int[W * H];
        for (int s = 0; s < W * H; s++) {
            int k = box[s];
            if (k == 0 || seen[s]) continue;
            int top = 0, size = 0;
            stack[top++] = s;
            seen[s] = true;
            while (top > 0) {
                int p = stack[--top];
                group[size++] = p;
                int pc = p % W, pr = p / W;
                for (int dr = -1; dr <= 1; dr++) {
                    for (int dc = -1; dc <= 1; dc++) {
                        int c = pc + dc, r = pr + dr, q = r * W + c;
                        if (c >= 0 && c < W && r >= 0 && r < H && !seen[q] && box[q] == k) {
                            seen[q] = true;
                            stack[top++] = q;
                        }
                    }
                }
            }
            if (size >= BUNCH) {
                for (int i = 0; i < size; i++) pop[group[i]] = true;
                n += size;
            }
        }
        return n;
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((h - top - 12) / H, (w - 70) / W);
        int ox = 8, oy = top + (h - top - s * H) / 2;
        g.setColor(SLAT);
        g.fillRoundRect(ox - 4, oy - 4, s * W + 8, s * H + 8, 10, 10);
        g.setColor(CRATE);
        g.fillRect(ox, oy, s * W, s * H);
        g.setColor(SLAT);
        for (int c = 1; c < W; c++) g.drawLine(ox + c * s, oy, ox + c * s, oy + s * H - 1);
        for (int i = 0; i < W * H; i++) {
            int cx = ox + (i % W) * s, cy = oy + (i / W) * s;
            if (pop[i]) burst(g, box[i], cx, cy, s);
            else if (box[i] != 0) fruit(g, box[i], cx + 1, cy + 1, s - 2);
        }
        if (state == PLAY && phase == FALL) {
            fruit(g, a, ox + x * s + 1, oy + y * s + 1, s - 2);
            fruit(g, b, ox + (x + DX[dir]) * s + 1, oy + (y + DY[dir]) * s + 1, s - 2);
        }

        int sx = ox + s * W + 10, sw = w - sx - 6, px = sx + sw / 2;
        int step = lineH + 6;
        g.setColor(WHITE);
        g.fillRoundRect(sx, oy - 4, sw, 2 * s + lineH + 18, 12, 12);
        g.setColor(SUB);
        g.drawString("다음", px, oy + 2, Graphics.TOP | Graphics.HCENTER);
        int fs = Math.min(s - 2, sw - 12);
        fruit(g, nextB, px - fs / 2, oy + lineH + 8, fs);
        fruit(g, nextA, px - fs / 2, oy + lineH + 10 + fs, fs);

        int iy = oy + 2 * s + lineH + 22;
        g.setColor(chain > 1 ? TEXT : WHITE);
        g.fillRoundRect(sx, iy, sw, 2 * step + 8, 12, 12);
        g.setColor(chain > 1 ? WHITE : SUB);
        g.drawString("연쇄", px, iy + 4, Graphics.TOP | Graphics.HCENTER);
        bold(g, "" + chain, px, iy + 4 + step, Graphics.TOP | Graphics.HCENTER);

        iy += 2 * step + 16;
        g.setColor(WHITE);
        g.fillRoundRect(sx, iy, sw, 4 * step + 8, 12, 12);
        g.setColor(SUB);
        g.drawString("최고", px, iy + 4, Graphics.TOP | Graphics.HCENTER);
        g.setColor(TEXT);
        g.drawString("" + bestChain, px, iy + 4 + step, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("느긋", px, iy + 4 + 2 * step, Graphics.TOP | Graphics.HCENTER);
        g.setColor(easy > 0 ? LEAF : TEXT);
        bold(g, "" + easy, px, iy + 4 + 3 * step, Graphics.TOP | Graphics.HCENTER);
    }

    /** A popping fruit: its colour as a ring around a pale core, one frame per step. */
    private static void burst(Graphics g, int k, int x, int y, int s) {
        g.setColor(COLORS[k]);
        g.fillArc(x - 2, y - 2, s + 4, s + 4, 0, 360);
        g.setColor(0xFFF8E6);
        g.fillArc(x + 4, y + 4, s - 8, s - 8, 0, 360);
    }

    /** One fruit in a d x d cell, each kind with its own mark. */
    private static void fruit(Graphics g, int k, int x, int y, int d) {
        int m = d / 2;
        g.setColor(COLORS[k]);
        if (k == 3) {
            int r = d * 11 / 20;
            g.fillArc(x, y + d - r - 1, r, r, 0, 360);
            g.fillArc(x + d - r, y + d - r - 1, r, r, 0, 360);
            g.fillArc(x + m - r / 2, y + 2, r, r, 0, 360);
            g.setColor(LEAF);
            g.fillRect(x + m - 1, y, 2, 3);
            return;
        }
        if (k == 1) {
            g.fillRoundRect(x + 1, y + 3, d - 2, d - 3, m + 2, d - 2);
            g.setColor(LEAF);
            g.fillRoundRect(x + m - 4, y + 1, 8, 4, 4, 4);
            g.setColor(WHITE);
            g.fillRect(x + m - 4, y + m + 1, 2, 2);
            g.fillRect(x + m + 2, y + m + 1, 2, 2);
            g.fillRect(x + m - 1, y + m + 5, 2, 2);
            return;
        }
        int inset = k == 5 ? d / 8 : 1;
        g.fillArc(x + inset, y + inset + 1, d - 2 * inset, d - 2 * inset - 1, 0, 360);
        g.setColor(WHITE);
        g.fillArc(x + inset + d / 5, y + inset + d / 5, 4, 4, 0, 360);
        if (k == 2) {
            g.setColor(LEAF);
            g.fillArc(x + m, y, d / 3, d / 5 + 2, 0, 360);
        } else if (k == 4) {
            g.setColor(0x8A6A4A);
            g.fillRect(x + m - 1, y, 2, d / 4 + 1);
            g.setColor(LEAF);
            g.fillArc(x + m + 1, y + 1, d / 4, d / 6 + 1, 0, 360);
        } else {
            g.setColor(0x4D6BA8);
            int cy = y + inset + 3;
            g.drawLine(x + m - 3, cy, x + m + 3, cy);
            g.drawLine(x + m, cy - 3, x + m, cy + 3);
        }
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int drop = (frame % 6) * 3;
        fruit(g, 1, cx - 9, cy - 26 + drop, 18);
        fruit(g, 1, cx - 29, cy + 6, 18);
        fruit(g, 1, cx - 9, cy + 6, 18);
        fruit(g, 3, cx + 11, cy + 6, 18);
        fruit(g, 2, cx + 31, cy + 6, 18);
    }
}
