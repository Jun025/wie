/*
 * The time a pop gives back, checked on the real game code (../../arcade-common/build-game.sh runs this
 * after building the jar, on the stubs, so a red check fails the build):
 * (1) a match gives time back (2) never past the full bar (3) a chain gives back more than one match.
 * Then a bot plays whole rounds at full motion, at three speeds, and prints how long they last.
 */
public class RechargeCheck {
    private static int fails;

    private static void check(boolean ok, String what) {
        System.out.println((ok ? "ok   " : "FAIL ") + what);
        if (!ok) fails++;
    }

    /** A round at 보통 with {@code left} ms on the clock, motion 끔. */
    private static MatchGame round(int left, int seed) {
        MatchGame g = new MatchGame();
        g.random.setSeed(seed);
        g.newGame(1);
        g.state = Arcade.PLAY;
        g.roundMs = 60000;
        g.leftMs = left;
        g.motion = 2;
        return g;
    }

    /**
     * The board (x + 2y) % 6 has no line; row 2 then reads 5 5 3 5, so swapping (2,2) with (3,2) makes
     * 5 5 5 there and leaves a 3 at (3,2). With {@code chain}, row 1 reads 2 3 3: after the drop row 2
     * starts 2 3 3 3, a second step. Returns the game after the swap, all of it resolved.
     */
    private static MatchGame swapOnce(boolean chain, int left) {
        MatchGame g = round(left, 1);
        byte[] b = new byte[3 + 49];
        b[0] = 6;
        b[1] = b[2] = 2;
        for (int i = 0; i < 49; i++) b[3 + i] = (byte) ((i % 7 + 2 * (i / 7)) % 6);
        b[3 + 14] = b[3 + 15] = b[3 + 17] = 5;
        b[3 + 16] = 3;
        if (chain) b[3 + 8] = b[3 + 9] = 3;
        check(g.restore(b, 0, b.length), "board loads");
        g.key(Arcade.FIRE);
        g.key(Arcade.RIGHT);
        return g;
    }

    /** One round at full motion: the bot swaps the first move it finds after {@code think} idle frames. */
    private static int roundMs(int seed, int think) {
        MatchGame g = round(60000, seed);
        g.motion = 0;
        int frames = 0, waited = 0;
        while (g.leftMs > 0 && frames < 100000) {
            frames++;
            g.leftMs -= 100; // one play loop turn (Arcade.run sleeps 100 ms)
            g.tick();
            if (g.phase != 0 || ++waited <= think) continue;
            waited = 0;
            int m = g.findMove();
            if (m < 0) continue;
            g.cx = m % 7;
            g.cy = m / 7;
            g.key(Arcade.FIRE);
            g.key(g.hintB == m + 1 ? Arcade.RIGHT : Arcade.DOWN);
        }
        return frames * 100;
    }

    public static void main(String[] args) {
        int single = swapOnce(false, 30000).leftMs - 30000;
        MatchGame c = swapOnce(true, 30000);
        int chain = c.leftMs - 30000;
        check(single > 0, "(1) one match of 3 gives time back: +" + single + " ms");
        check(swapOnce(false, 30000).score == 30, "    that was one step (score 30)");
        MatchGame full = swapOnce(false, 59900);
        check(full.leftMs <= full.roundMs, "(2) never past the full bar: " + full.leftMs + " of " + full.roundMs);
        check(c.score >= 90, "    the chain case chained (score " + c.score + ")");
        check(chain > single, "(3) a chain gives back more: +" + chain + " ms > +" + single + " ms");
        // Rounds are 60 s without the refill. 2 frames = a swap every 0.2 s of idle board, far faster than a thumb.
        int[] thinks = {20, 10, 2};
        for (int t = 0; t < thinks.length; t++) {
            int sum = 0, max = 0;
            for (int seed = 1; seed <= 20; seed++) {
                int ms = roundMs(seed, thinks[t]);
                sum += ms;
                max = Math.max(max, ms);
            }
            System.out.println("     bot, a swap after " + thinks[t] * 100 + " ms idle: round " + sum / 20 + " ms on average, " + max + " ms at most (20 seeds)");
            check(max < 2 * 60000, "    a round stays under twice its length");
        }
        if (fails > 0) {
            System.out.println(fails + " FAIL");
            System.exit(1);
        }
    }
}
