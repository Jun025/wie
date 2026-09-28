/*
 * 쏙쏙 두더지 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Nine holes laid out like the keypad: key 1 is the top-left hole, 9 the bottom-right.
 * Tap a mole while it is up. A gold one is worth three; the prickly hedgehog costs
 * points. 30 seconds a round, and the field speeds up as it goes — but a critter
 * always stays up for at least 850 ms, a few frames even when the engine runs at 4 fps.
 */

import javax.microedition.lcdui.Graphics;

public class SsokssokMole extends Arcade.Midlet {
    Arcade create() {
        return new MoleGame();
    }
}

final class MoleGame extends Arcade {
    private static final int EMPTY = 0, MOLE = 1, GOLD = 2, HEDGEHOG = 3, ROUND_MS = 30000;
    private static final int GROUND = 0xE8DFD2, HOLE = 0x6B5646, FUR = 0xB88A6A, GOLD_FUR = 0xF2C766,
        SPIKE = 0x8C7B6B, NOSE = 0xF29B97;

    private final int[] who = new int[9];
    private final long[] downAt = new long[9];
    private final long[] bonkUntil = new long[9];
    private long startAt, endAt, nextSpawn;

    String name() {
        return "쏙쏙 두더지";
    }

    String tagline() {
        return "올라온 두더지를 톡!";
    }

    String[] guide() {
        return new String[] {"1 ~ 9 : 그 자리 구멍을 톡", "노란 두더지는 30점", "고슴도치는 치지 마세요"};
    }

    void newGame() {
        for (int i = 0; i < 9; i++) who[i] = EMPTY;
        startAt = System.currentTimeMillis();
        endAt = startAt + ROUND_MS;
        nextSpawn = startAt + 600;
    }

    String status() {
        long left = state == PLAY ? Math.max(0, endAt - System.currentTimeMillis()) : 0;
        return (left + 999) / 1000 + "초";
    }

    int delay() {
        return 100;
    }

    void tick(long now) {
        if (now >= endAt) {
            gameOver(false);
            return;
        }
        for (int i = 0; i < 9; i++) if (who[i] != EMPTY && now >= downAt[i]) who[i] = EMPTY;
        if (now >= nextSpawn) {
            // 0 at the start of the round, 1000 at the end.
            int t = (int) ((now - startAt) * 1000 / ROUND_MS);
            int i = rnd(9);
            if (who[i] == EMPTY && now >= bonkUntil[i]) {
                int r = rnd(10);
                who[i] = r == 0 ? GOLD : r <= 2 && t > 200 ? HEDGEHOG : MOLE;
                downAt[i] = now + 1500 - 650 * t / 1000;
            }
            nextSpawn = now + 850 - 400 * t / 1000;
        }
    }

    void key(int action, int key) {
        if (key < KEY_NUM1 || key > KEY_NUM9) return;
        int i = key - KEY_NUM1;
        int w = who[i];
        if (w == EMPTY) return;
        who[i] = EMPTY;
        bonkUntil[i] = System.currentTimeMillis() + 300;
        if (w == HEDGEHOG) {
            score = Math.max(0, score - 20);
            toast("앗, 따가워! -20");
            sound.play(Sound.BAD);
        } else {
            score += w == GOLD ? 30 : 10;
            sound.play(w == GOLD ? Sound.BIG : Sound.GOOD);
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int cell = Math.min((w - 20) / 3, (h - top - 20) / 3);
        int ox = (w - cell * 3) / 2, oy = top + (h - top - cell * 3) / 2;
        g.setColor(GROUND);
        g.fillRoundRect(ox - 6, oy - 6, cell * 3 + 12, cell * 3 + 12, 22, 22);
        long now = System.currentTimeMillis();
        for (int i = 0; i < 9; i++) {
            int x = ox + (i % 3) * cell, y = oy + (i / 3) * cell;
            int hw = cell - 16, hh = cell / 3;
            int hx = x + 8, hy = y + cell - hh - 8;
            if (now < bonkUntil[i]) {
                g.setColor(0xFFF3C4);
                g.fillArc(hx - 6, hy - 12, hw + 12, hh + 18, 0, 360);
            }
            g.setColor(HOLE);
            g.fillArc(hx, hy, hw, hh, 0, 360);
            if (who[i] != EMPTY) {
                critter(g, who[i], x + cell / 2, hy + hh / 2, hw * 3 / 4);
                g.setColor(GROUND); // the near rim hides the critter's lower half
                g.fillArc(hx - 2, hy + hh / 2, hw + 4, hh + 2, 180, 180);
                g.setColor(HOLE);
                g.fillArc(hx, hy + hh / 4, hw, hh / 2, 180, 180);
            }
            g.setColor(SUB);
            g.drawString("" + (i + 1), x + 6, y + 4, Graphics.TOP | Graphics.LEFT);
        }
    }

    /** A critter peeking out of a hole whose middle is (cx, base). */
    private static void critter(Graphics g, int kind, int cx, int base, int d) {
        int x = cx - d / 2, y = base - d;
        if (kind == HEDGEHOG) {
            g.setColor(SPIKE);
            for (int a = -3; a <= 3; a++) g.drawLine(cx, base - d / 2, cx + a * d / 6, y - 4 + Math.abs(a) * 3);
        }
        g.setColor(kind == GOLD ? GOLD_FUR : kind == HEDGEHOG ? SPIKE : FUR);
        g.fillRoundRect(x, y, d, d + 4, d / 2, d / 2);
        g.setColor(kind == HEDGEHOG ? 0xE8DFD2 : 0xF6EBDD);
        g.fillArc(x + d / 5, y + d / 3, d * 3 / 5, d / 2, 0, 360);
        g.setColor(TEXT);
        g.fillRect(cx - d / 5, y + d / 4, 3, 3);
        g.fillRect(cx + d / 5 - 2, y + d / 4, 3, 3);
        g.setColor(NOSE);
        g.fillArc(cx - 3, y + d / 3 + 2, 7, 5, 0, 360);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        g.setColor(HOLE);
        g.fillArc(cx - 30, cy + 6, 60, 16, 0, 360);
        critter(g, (frame / 6) % 3 == 2 ? GOLD : MOLE, cx, cy + 14, 34 - ((frame / 2) % 2) * 4);
    }
}
