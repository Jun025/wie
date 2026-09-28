/*
 * 뽁뽁 풍선 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * A duck on a lake flicks pebbles up at a block of balloons that sways over the water and
 * sinks one row at a time. Balloons now and then let go of a water drop; three drops on
 * the duck, or the balloons reaching the water, end the round. Clear the sky and the next
 * one comes in lower and quicker.
 *
 * The rule this game is built around: each balloon row is one colour, and popping the
 * SAME colour again right after multiplies the points (x2 … x5). So the good play is to
 * slide along a row, not to dig one column — a different colour or a pebble that flies
 * off the top resets it.
 *
 * Everything moves on a grid, one cell per frame at most, and every speed is counted in
 * frames, never milliseconds. At 4 fps the game is slower, not harder, and a pebble
 * cannot skip past a balloon: hits are checked after the pebble moves AND after the
 * balloons move, so the two can never swap cells unseen.
 */

import javax.microedition.lcdui.Graphics;

public class BbokBalloon extends Arcade.Midlet {
    Arcade create() {
        return new BalloonGame();
    }
}

final class BalloonGame extends Arcade {
    private static final int COLS = 9, ROWS = 13, FR = 4, FC = 7, SHOTS = 2, DROPS = 3;
    private static final int[] COLORS = {0, 0xF4A0B5, 0x8FD0B0, 0x92B8F0, 0xF3CC6A};
    private static final int SKY = 0xE3F1FB, WATER = 0xC7E2F4;

    private final int[] bal = new int[FR * FC];
    private final int[] sx = new int[SHOTS], sy = new int[SHOTS];
    private final int[] dx = new int[DROPS], dy = new int[DROPS];
    private int fx, fy, sway, wave, lives, duck, t, mult, lastColor, hurtUntil, popAt = -9, popX, popY;

    String name() {
        return "뽁뽁 풍선";
    }

    String tagline() {
        return "같은 색을 이어 터뜨려요";
    }

    String[] guide() {
        return new String[] {"4 6 / 좌우 : 오리 옮기기", "5 : 조약돌 던지기", "같은 색 연달아 = 점수 x2 x3", "물방울은 피해요 · 목숨 셋"};
    }

    int delay() {
        return 120;
    }

    String status() {
        return "하늘 " + wave + " · 목숨 " + lives;
    }

    void newGame() {
        wave = 0;
        lives = 3;
        duck = COLS / 2;
        nextWave();
    }

    private void nextWave() {
        wave++;
        for (int r = 0; r < FR; r++) {
            int k = 1 + rnd(4);
            for (int c = 0; c < FC; c++) bal[r * FC + c] = k;
        }
        fx = 1;
        fy = 1;
        sway = 1;
        t = 0;
        hurtUntil = 0;
        popAt = -9;
        mult = 1;
        lastColor = 0;
        for (int i = 0; i < SHOTS; i++) sy[i] = -1;
        for (int i = 0; i < DROPS; i++) dy[i] = -1;
        if (wave > 1) toast("하늘 " + wave);
    }

    // Difficulty lives in frame counts, so a slow handset plays the same game, slower.
    private int sinkEvery() {
        return Math.max(10, 30 - wave * 4);
    }

    private int swayEvery() {
        return Math.max(3, 7 - wave);
    }

    private int dropOdds() {
        return Math.max(5, 18 - wave * 3);
    }

    void key(int action, int key) {
        if (action == LEFT && duck > 0) duck--;
        else if (action == RIGHT && duck < COLS - 1) duck++;
        else if (action == FIRE) {
            for (int i = 0; i < SHOTS; i++) {
                if (sy[i] < 0) {
                    sx[i] = duck;
                    sy[i] = ROWS - 2;
                    hitShot(i);
                    return;
                }
            }
        }
        hitDuck();
    }

    void tick(long now) {
        t++;
        for (int i = 0; i < SHOTS; i++) {
            if (sy[i] < 0) continue;
            if (--sy[i] < 0) {
                mult = 1; // flew off the top: the colour run is broken
                lastColor = 0;
            } else {
                hitShot(i);
            }
        }
        if (t % swayEvery() == 0) {
            if (fx + sway < 0 || fx + sway > COLS - FC) sway = -sway;
            fx += sway;
        }
        if (t % sinkEvery() == 0) fy++;
        for (int i = 0; i < SHOTS; i++) if (sy[i] >= 0) hitShot(i);
        if (lowest() >= ROWS - 1) {
            gameOver(false);
            return;
        }
        if (t % 2 == 0) {
            for (int i = 0; i < DROPS; i++) {
                if (dy[i] >= 0 && ++dy[i] >= ROWS) dy[i] = -1;
            }
        }
        if (rnd(dropOdds()) == 0) letGo();
        hitDuck();
        if (lowest() < 0) nextWave();
    }

    /** The screen row of the lowest balloon still up, or -1 when the sky is clear. */
    private int lowest() {
        for (int r = FR - 1; r >= 0; r--) {
            for (int c = 0; c < FC; c++) if (bal[r * FC + c] != 0) return fy + r;
        }
        return -1;
    }

    private void hitShot(int i) {
        int c = sx[i] - fx, r = sy[i] - fy;
        if (c < 0 || c >= FC || r < 0 || r >= FR || bal[r * FC + c] == 0) return;
        int k = bal[r * FC + c];
        bal[r * FC + c] = 0;
        sy[i] = -1;
        mult = k == lastColor ? Math.min(5, mult + 1) : 1;
        lastColor = k;
        score += 10 * mult;
        popAt = t;
        popX = sx[i];
        popY = fy + r;
        if (mult > 1) {
            toast("같은 색 x" + mult);
            sound.play(Sound.BIG);
        } else {
            sound.play(Sound.GOOD);
        }
    }

    /** Half the drops aim at the duck's column, so standing still is never safe. */
    private void letGo() {
        int slot = -1;
        for (int i = 0; i < DROPS; i++) if (dy[i] < 0) slot = i;
        if (slot < 0) return;
        int col = rnd(2) == 0 ? duck : fx + rnd(FC);
        int c = col - fx;
        if (c < 0 || c >= FC) return;
        for (int r = FR - 1; r >= 0; r--) {
            if (bal[r * FC + c] != 0) {
                dx[slot] = col;
                dy[slot] = fy + r + 1;
                return;
            }
        }
    }

    private void hitDuck() {
        if (state != PLAY || t < hurtUntil) return;
        for (int i = 0; i < DROPS; i++) {
            if (dy[i] == ROWS - 1 && dx[i] == duck) {
                for (int j = 0; j < DROPS; j++) dy[j] = -1;
                hurtUntil = t + 8;
                mult = 1;
                lastColor = 0;
                if (--lives == 0) {
                    gameOver(false);
                } else {
                    toast("앗, 물방울!");
                    sound.play(Sound.BAD);
                }
                return;
            }
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 16) / COLS, (h - top - 12) / ROWS);
        int ox = (w - s * COLS) / 2, oy = top + (h - top - s * ROWS) / 2;
        g.setColor(SKY);
        g.fillRoundRect(ox - 4, oy - 4, s * COLS + 8, s * ROWS + 8, 16, 16);
        g.setColor(WATER);
        g.fillRoundRect(ox - 4, oy + s * (ROWS - 1) - 2, s * COLS + 8, s + 6, 16, 16);
        g.setColor(WHITE);
        g.fillArc(ox + s, oy + s / 2, 3 * s, s, 0, 360); // one cloud, for the daylight
        g.fillArc(ox + 2 * s, oy, 2 * s, s + s / 2, 0, 360);

        for (int r = 0; r < FR; r++) {
            for (int c = 0; c < FC; c++) {
                int k = bal[r * FC + c];
                if (k != 0) balloon(g, k, ox + (fx + c) * s, oy + (fy + r) * s, s);
            }
        }
        if (t - popAt < 2) {
            g.setColor(0xFFF3C4);
            g.fillArc(ox + popX * s, oy + popY * s, s, s, 0, 360);
        }
        g.setColor(0x7D8794);
        for (int i = 0; i < SHOTS; i++) {
            if (sy[i] >= 0) g.fillArc(ox + sx[i] * s + s / 2 - 3, oy + sy[i] * s + s / 2 - 3, 7, 6, 0, 360);
        }
        for (int i = 0; i < DROPS; i++) {
            if (dy[i] < 0) continue;
            int x = ox + dx[i] * s + s / 2, y = oy + dy[i] * s + s / 2;
            g.setColor(0x5E9ED8);
            g.fillArc(x - 4, y - 2, 9, 9, 0, 360);
            g.fillRect(x - 1, y - 6, 3, 5);
        }
        if (t >= hurtUntil || t % 2 == 0) duck(g, ox + duck * s, oy + (ROWS - 1) * s, s);
    }

    private static void balloon(Graphics g, int k, int x, int y, int s) {
        int d = s - 4;
        g.setColor(SUB);
        g.drawLine(x + s / 2, y + d, x + s / 2 - 1, y + s + 2);
        g.setColor(COLORS[k]);
        g.fillArc(x + 2, y, d, d, 0, 360);
        mark(g, k, x + 2 + d / 2, y + d / 2);
    }

    /** A white mark per colour (dot, dash, bar, none), so colour is not the only cue. */
    private static void mark(Graphics g, int k, int cx, int cy) {
        g.setColor(WHITE);
        if (k == 1) g.fillArc(cx - 2, cy - 2, 5, 5, 0, 360);
        else if (k == 2) g.fillRect(cx - 4, cy - 1, 8, 2);
        else if (k == 3) g.fillRect(cx - 1, cy - 4, 2, 8);
    }

    private static void duck(Graphics g, int x, int y, int s) {
        g.setColor(0xF6C85F);
        g.fillArc(x, y + s / 3, s, s * 2 / 3, 0, 360);
        g.fillArc(x + s / 4, y, s / 2 + 2, s / 2 + 2, 0, 360);
        g.setColor(0xF0915A);
        g.fillRoundRect(x + s * 3 / 4, y + s / 5, s / 4 + 2, s / 6 + 2, 4, 4);
        g.setColor(TEXT);
        g.fillRect(x + s / 2 + 1, y + s / 6, 2, 2);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int bob = (frame % 4 < 2) ? 0 : 2;
        balloon(g, 1, cx - 34, cy - 22 + bob, 22);
        balloon(g, 1, cx - 12, cy - 24 - bob, 22);
        balloon(g, 3, cx + 10, cy - 22 + bob, 22);
        duck(g, cx - 11, cy + 6, 22);
    }
}
