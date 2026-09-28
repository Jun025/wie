/*
 * 통통 조약돌 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * A pebble bounces off a wooden plank and chips away a picture made of pastel tiles —
 * a tulip, then a little house, then a fish. Clear the picture to move on; clear all
 * three and the round is won. Miss the pebble three times and it is over.
 *
 * The rule this game is built around: a tile that breaks also breaks the tiles of the
 * same colour right next to it (up, down, left, right — one step, not the whole patch),
 * and the points grow with how many went together. Aiming at the middle of a colour
 * patch pays; nibbling its edge does not.
 *
 * Where the pebble lands on the plank steers it: left third sends it left, right third
 * right, the middle keeps its line. It moves one cell per frame — diagonally or straight
 * up and down — and every bounce is decided before the move, so at 4 fps it never passes
 * through a tile or the plank; the game only runs slower.
 */

import javax.microedition.lcdui.Graphics;

public class TongtongPebble extends Arcade.Midlet {
    Arcade create() {
        return new PebbleGame();
    }
}

final class PebbleGame extends Arcade {
    private static final int COLS = 11, ROWS = 17, PLANK = 3, WAIT = 20;
    private static final int[] COLORS = {0, 0xF4A7C0, 0x9ED39A, 0x8FC3EE, 0xF5D06F};
    // Three pictures of our own, 11 wide; a digit is a tile colour, '.' is empty.
    private static final String[][] PICTURES = {
        {"...11.11...", "..1111111..", "..1111111..", "...11111...", ".....2.....", "..22.2.22..", "...22222..."},
        {".....4.....", "....444....", "...44444...", "..4444444..", "..1111111..", "..1331331..", "..1111221.."},
        {"...........", "...3333..22", ".3133333322", ".3333333322", "...3333..22", ".4.......4.", "4.4.....4.."},
    };
    private static final String[] PICTURE_NAMES = {"튤립", "집", "물고기"};

    private final int[] tiles = new int[COLS * ROWS];
    private final boolean[] spark = new boolean[COLS * ROWS];
    private int stage, lives, plank, bx, by, dx, dy, held, t, sparkAt = -9;

    String name() {
        return "통통 조약돌";
    }

    String tagline() {
        return "그림 타일을 톡톡 깨요";
    }

    String[] guide() {
        return new String[] {"4 6 / 좌우 : 판자 옮기기", "5 : 조약돌 띄우기", "판자 끝에 맞으면 옆으로", "옆의 같은 색 타일도 함께 깨져요"};
    }

    int delay() {
        return 110;
    }

    String status() {
        return PICTURE_NAMES[stage] + " · 목숨 " + lives;
    }

    void newGame() {
        stage = 0;
        lives = 3;
        loadPicture();
    }

    private void loadPicture() {
        for (int i = 0; i < COLS * ROWS; i++) tiles[i] = 0;
        String[] rows = PICTURES[stage];
        for (int r = 0; r < rows.length; r++) {
            for (int c = 0; c < COLS; c++) {
                char ch = rows[r].charAt(c);
                if (ch != '.') tiles[(r + 1) * COLS + c] = ch - '0';
            }
        }
        plank = (COLS - PLANK) / 2;
        hold();
    }

    /** The pebble rests on the plank until 5, or until it lifts off by itself. */
    private void hold() {
        held = WAIT;
        bx = plank + 1;
        by = ROWS - 2;
        dx = rnd(2) == 0 ? -1 : 1;
        dy = -1;
    }

    void key(int action, int key) {
        if (action == LEFT) plank = Math.max(0, plank - 2);
        else if (action == RIGHT) plank = Math.min(COLS - PLANK, plank + 2);
        else if (action == FIRE && held > 0) held = 1;
        if (held > 0) bx = plank + 1;
    }

    void tick(long now) {
        t++;
        if (held > 0) {
            held--;
            return;
        }
        if (dx != 0) {
            int nx = bx + dx;
            if (nx < 0 || nx >= COLS) dx = -dx;
            else if (tile(nx, by)) {
                breakAt(nx, by);
                dx = -dx;
            }
        }
        int ny = by + dy;
        if (ny < 0) dy = 1;
        else if (tile(bx, ny)) {
            breakAt(bx, ny);
            dy = -dy;
        } else if (dx != 0 && tile(bx + dx, ny)) {
            breakAt(bx + dx, ny); // a corner: the pebble comes straight back
            dx = -dx;
            dy = -dy;
        }
        if (dy == 1 && by + 1 == ROWS - 1) {
            int at = Math.max(0, Math.min(COLS - 1, bx + dx));
            if (at >= plank && at < plank + PLANK || bx >= plank && bx < plank + PLANK) {
                int part = at - plank;
                if (part <= 0) dx = -1;
                else if (part >= PLANK - 1) dx = 1;
                dy = -1;
                sound.play(Sound.MOVE);
            }
        }
        int nx = bx + dx;
        ny = by + dy;
        if (nx >= 0 && nx < COLS && ny >= 0 && !tile(nx, ny)) {
            bx = nx;
            by = ny;
        }
        if (by >= ROWS - 1) {
            miss();
            return;
        }
        if (left() == 0) {
            score += 50;
            if (++stage == PICTURES.length) {
                score += lives * 100;
                stage--;
                gameOver(true);
            } else {
                toast(PICTURE_NAMES[stage] + " 차례");
                sound.play(Sound.BIG);
                loadPicture();
            }
        }
    }

    private void miss() {
        if (--lives == 0) {
            gameOver(false);
            return;
        }
        toast("놓쳤어요 · 목숨 " + lives);
        sound.play(Sound.BAD);
        hold();
    }

    private boolean tile(int c, int r) {
        return c >= 0 && c < COLS && r >= 0 && r < ROWS && tiles[r * COLS + c] != 0;
    }

    private int left() {
        int n = 0;
        for (int i = 0; i < COLS * ROWS; i++) if (tiles[i] != 0) n++;
        return n;
    }

    /** Breaks one tile and its same-colour neighbours one step away; more together, more points. */
    private void breakAt(int c, int r) {
        for (int i = 0; i < COLS * ROWS; i++) spark[i] = false;
        int k = tiles[r * COLS + c];
        int n = 0;
        int[] ox = {0, -1, 1, 0, 0}, oy = {0, 0, 0, -1, 1};
        for (int j = 0; j < 5; j++) {
            int cc = c + ox[j], rr = r + oy[j];
            if (tile(cc, rr) && tiles[rr * COLS + cc] == k) {
                tiles[rr * COLS + cc] = 0;
                spark[rr * COLS + cc] = true;
                n++;
            }
        }
        score += 10 * n * n;
        sparkAt = t;
        if (n >= 3) {
            toast(n + "개 함께!");
            sound.play(Sound.BIG);
        } else {
            sound.play(Sound.GOOD);
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 16) / COLS, (h - top - 12) / ROWS);
        int ox = (w - s * COLS) / 2, oy = top + (h - top - s * ROWS) / 2;
        g.setColor(WHITE);
        g.fillRoundRect(ox - 4, oy - 4, s * COLS + 8, s * ROWS + 8, 16, 16);
        boolean sparkle = t - sparkAt < 3;
        for (int i = 0; i < COLS * ROWS; i++) {
            int x = ox + (i % COLS) * s, y = oy + (i / COLS) * s;
            if (tiles[i] != 0) {
                g.setColor(COLORS[tiles[i]]);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, s / 3, s / 3);
                mark(g, tiles[i], x + s / 2, y + s / 2);
            } else if (sparkle && spark[i]) {
                g.setColor(0xFFF3C4);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, s / 3, s / 3);
            }
        }
        int py = oy + (ROWS - 1) * s;
        g.setColor(0xD9B48A);
        g.fillRoundRect(ox + plank * s + 1, py + s / 4, PLANK * s - 2, s / 2, s / 2, s / 2);
        g.setColor(0xB98E62);
        g.fillRect(ox + plank * s + s / 2, py + s / 2 - 1, PLANK * s - s, 2);
        pebble(g, ox + bx * s + s / 2, oy + by * s + s / 2, s * 2 / 3);
        if (held > 0 && state == PLAY && (t / 2) % 2 == 0) {
            g.setColor(SUB);
            g.drawString("5 : 띄우기", w / 2, oy + (ROWS - 5) * s, Graphics.TOP | Graphics.HCENTER);
        }
    }

    /** A white mark per colour (dot, dash, bar, none), so colour is not the only cue. */
    private static void mark(Graphics g, int k, int cx, int cy) {
        g.setColor(WHITE);
        if (k == 1) g.fillArc(cx - 2, cy - 2, 5, 5, 0, 360);
        else if (k == 2) g.fillRect(cx - 4, cy - 1, 8, 2);
        else if (k == 3) g.fillRect(cx - 1, cy - 4, 2, 8);
    }

    private static void pebble(Graphics g, int cx, int cy, int d) {
        g.setColor(0x8B95A3);
        g.fillArc(cx - d / 2, cy - d / 2 + 1, d, d - 2, 0, 360);
        g.setColor(0xC9D0D9);
        g.fillArc(cx - d / 4, cy - d / 3, d / 3, d / 4, 0, 360);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int[] ks = {1, 2, 3, 4, 1};
        for (int i = 0; i < 5; i++) {
            g.setColor(COLORS[ks[i]]);
            g.fillRoundRect(cx - 50 + i * 20, cy - 22, 18, 12, 8, 8);
        }
        int hop = (frame % 6 < 3) ? 0 : 6;
        pebble(g, cx + 6, cy - hop, 12);
        g.setColor(0xD9B48A);
        g.fillRoundRect(cx - 20, cy + 12, 40, 8, 8, 8);
    }
}
