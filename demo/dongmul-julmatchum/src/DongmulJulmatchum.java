/*
 * 동물줄맞춤 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Swap two neighbouring animal faces so three or more of one kind line up; they pop, the
 * faces above drop, and whatever lines up after the drop pops again for a bigger
 * multiplier. The round ends when the time bar runs out. Each animal differs by its
 * silhouette (ears) as well as its colour, so the board still reads without colour.
 * The faces are PNGs drawn at build time by ../art/Faces.java — see why there.
 * How the board animates, and why input never waits for it: the comment above IDLE below.
 */

import javax.microedition.lcdui.Font;
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
    /** Each face's rim colour (art/Faces.java) — the pop ring, which must show on a white cell. */
    private static final int[] RIMS = {0xAE7A30, 0x5F3F2D, 0x5A6A88, 0x958AB8, 0x252D40, 0x55935B};

    /*
     * The board moves in phases, each a whole number of loop frames — never milliseconds, so a
     * slow device sees the same animation, only slower, and the rules do not change with speed.
     * Per motion setting (켬, 줄임, 끔): a phase of 0 frames is skipped, so 끔 plays like the
     * board before animation: everything resolves inside the key press.
     * Input never waits for a phase: moving the cursor and picking work at any time, and a swap
     * pressed while the board still moves first completes the moving part at once (finish()).
     */
    private static final int IDLE = 0, SWAP = 1, NOSWAP = 2, POP = 3, FALL = 4;
    private static final int[] SWAP_F = {2, 0, 0}, NOSWAP_F = {4, 0, 0}, POP_F = {3, 1, 0};
    /** A refused swap leans toward its neighbour and back, in hundredths of a cell per frame. */
    private static final int[] WOBBLE = {30, 45, 20, -8};
    /** Frames a score or chain label stays up; frames of no input before the hint shows. */
    private static final int LABEL_F = 6, BADGE_F = 8, HINT_F = 60, FIRST_HINT_F = 30;

    private final Image[] small = new Image[6], big = new Image[6], happy = new Image[6], alert = new Image[6];
    private final int[] board = new int[N * N];
    private final boolean[] hit = new boolean[N * N];
    /** During FALL: how many cells each face still has to come down from. */
    private final int[] drop = new int[N * N];
    private int kinds = 6;
    private int cx, cy;
    private boolean picked;
    private int phase, f, dur, a, b, chain, maxDrop;
    private int idle, hintA = -1, hintB;
    // Rising score labels (position in hundredths of a cell), and the chain badge.
    private final int[] labelX = new int[4], labelY = new int[4], labelAge = new int[4];
    private final String[] labelText = new String[4];
    private int badgeAge = BADGE_F, badgeChain;

    MatchGame() {
        for (int k = 0; k < 6; k++) {
            small[k] = load("/f" + k + "_30.png");
            big[k] = load("/f" + k + "_56.png");
            happy[k] = load("/f" + k + "_30h.png");
            alert[k] = load("/f" + k + "_30a.png");
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
        reset();
        return ROUND_MS[level];
    }

    /** Cursor to the middle, nothing moving, no hint or labels up. */
    private void reset() {
        cx = cy = N / 2;
        picked = false;
        phase = IDLE;
        idle = chain = 0;
        hintA = -1;
        badgeAge = BADGE_F;
        for (int i = 0; i < labelAge.length; i++) labelAge[i] = LABEL_F;
        for (int i = 0; i < N * N; i++) drop[i] = 0;
    }

    /** kinds, cursor x, cursor y, then the 49 cells. The board is settled: pause calls end() first. */
    byte[] snapshot() {
        byte[] b = new byte[3 + N * N];
        b[0] = (byte) kinds;
        b[1] = (byte) cx;
        b[2] = (byte) cy;
        for (int i = 0; i < N * N; i++) b[3 + i] = (byte) board[i];
        return b;
    }

    boolean restore(byte[] b, int off, int len) {
        if (len != 3 + N * N || b[off] < 5 || b[off] > 6) return false;
        for (int i = 0; i < N * N; i++) if (b[off + 3 + i] < 0 || b[off + 3 + i] >= b[off]) return false;
        kinds = b[off];
        for (int i = 0; i < N * N; i++) board[i] = b[off + 3 + i];
        reset();
        cx = Math.min(N - 1, Math.max(0, b[off + 1]));
        cy = Math.min(N - 1, Math.max(0, b[off + 2]));
        if (findMove() < 0) fill();
        return true;
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
        } while (findMove() < 0);
    }

    void key(int action) {
        idle = 0;
        hintA = -1;
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
        finish();
        a = cy * N + cx;
        b = ny * N + nx;
        swap(a, b);
        if (findLines() == 0) {
            swap(a, b);
            sound.play(Sound.BAD);
            begin(NOSWAP, NOSWAP_F[motion]);
        } else {
            cx = nx;
            cy = ny;
            chain = 0;
            begin(SWAP, SWAP_F[motion]);
        }
        settle();
    }

    void tick() {
        for (int i = 0; i < labelAge.length; i++) if (labelAge[i] < LABEL_F) labelAge[i]++;
        if (badgeAge < BADGE_F) badgeAge++;
        if (phase == IDLE) {
            if (++idle == (best[level] == 0 ? FIRST_HINT_F : HINT_F)) hintA = findMove();
            return;
        }
        if (++f >= dur) endPhase();
        settle();
    }

    void end() {
        finish();
    }

    private void begin(int p, int frames) {
        phase = p;
        f = 0;
        dur = frames;
    }

    /** Runs every phase that has no frames at this motion setting. */
    private void settle() {
        while (phase != IDLE && dur == 0) endPhase();
    }

    /** Completes whatever is still moving, at once. */
    private void finish() {
        while (phase != IDLE) endPhase();
    }

    private void endPhase() {
        if (phase == SWAP) {
            pop();
        } else if (phase == POP) {
            collapse();
        } else if (phase == FALL) {
            for (int i = 0; i < N * N; i++) drop[i] = 0;
            if (findLines() > 0) {
                pop();
            } else if (findMove() < 0) {
                fill();
                for (int i = 0; i < N * N; i++) drop[i] = N;
                maxDrop = N;
                toast("판을 새로 섞었어요");
                begin(FALL, fallFrames());
            } else {
                phase = IDLE;
            }
        } else {
            phase = IDLE;
        }
    }

    /** The lines on the board start to pop: they score now, and vanish when the phase ends. */
    private void pop() {
        int popped = findLines();
        chain++;
        int gain = popped * 10 * chain;
        score += gain;
        int sx = 0, sy = 0;
        for (int i = 0; i < N * N; i++) {
            if (hit[i]) {
                sx += (i % N) * 100 + 50;
                sy += (i / N) * 100 + 50;
            }
        }
        int slot = 0;
        for (int i = 1; i < labelAge.length; i++) if (labelAge[i] > labelAge[slot]) slot = i;
        labelX[slot] = sx / popped;
        labelY[slot] = sy / popped;
        labelText[slot] = "+" + gain;
        labelAge[slot] = 0;
        if (chain > 1) {
            badgeChain = chain;
            badgeAge = 0;
            if (motion == 2) toast("연쇄 x" + chain);
            sound.play(Sound.BIG);
            buzz(60);
        } else {
            sound.play(Sound.GOOD);
        }
        begin(POP, POP_F[motion]);
    }

    /** Popped faces go; the faces above fall into the gaps and new ones come in from the top. */
    private void collapse() {
        maxDrop = 0;
        for (int x = 0; x < N; x++) {
            int to = N - 1;
            for (int y = N - 1; y >= 0; y--) {
                int i = y * N + x;
                if (hit[i]) continue;
                board[to * N + x] = board[i];
                drop[to * N + x] = to - y;
                to--;
            }
            for (int y = to; y >= 0; y--) {
                board[y * N + x] = rnd(kinds);
                drop[y * N + x] = to + 1;
            }
            maxDrop = Math.max(maxDrop, to + 1);
        }
        begin(FALL, fallFrames());
    }

    /** How far a falling face has come after {@code n} frames, in hundredths of a cell: it speeds up. */
    private static int travel(int n) {
        return 25 * n * (n + 1);
    }

    /** Frames until the longest drop lands, plus one for the landing dip. Only full motion falls. */
    private int fallFrames() {
        if (motion != 0) return 0;
        int n = 0;
        while (travel(n) < maxDrop * 100) n++;
        return n + 1;
    }

    private void swap(int p, int q) {
        int t = board[p];
        board[p] = board[q];
        board[q] = t;
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

    /** A cell whose swap with {@code hintB} makes a line, or -1 if the board has no move. */
    private int findMove() {
        for (int i = 0; i < N * N; i++) {
            if (i % N + 1 < N && swapMakesLine(i, i + 1)) {
                hintB = i + 1;
                return i;
            }
            if (i + N < N * N && swapMakesLine(i, i + N)) {
                hintB = i + N;
                return i;
            }
        }
        return -1;
    }

    private boolean swapMakesLine(int p, int q) {
        swap(p, q);
        boolean ok = findLines() > 0;
        swap(p, q);
        return ok;
    }

    /** A face centred on (x, y); {@code large} picks the 56px set over the 30px one. */
    private void face(Graphics g, int k, int x, int y, boolean large) {
        face(g, k, x, y, large ? big : small);
    }

    private void face(Graphics g, int k, int x, int y, Image[] set) {
        if (k < 0) return;
        Image img = set[k] != null ? set[k] : small[k];
        if (img != null) {
            g.drawImage(img, x, y, Graphics.HCENTER | Graphics.VCENTER);
        } else {
            int d = set == big ? 50 : 26;
            g.setColor(COLORS[k]);
            g.fillArc(x - d / 2, y - d / 2, d, d, 0, 360);
        }
    }

    void paintBoard(Graphics g, int top, int w, int h) {
        int s = Math.min((w - 16) / N, (h - top - 12) / N);
        int ox = (w - s * N) / 2, oy = top + (h - top - s * N) / 2;
        card(g, ox - 4, oy - 4, s * N + 8, s * N + 8, 18);
        // 켬 walks the pop through three looks (glad face, small ring, wide pale ring); 줄임 shows only the first.
        int popStage = phase != POP ? -1 : motion == 0 ? f : 0;
        for (int i = 0; i < N * N; i++) {
            int x = ox + (i % N) * s, y = oy + (i / N) * s;
            boolean popping = popStage >= 0 && hit[i];
            if (popStage == 0 && hit[i]) {
                g.setColor(0xFFF1BF);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, 12, 12);
            } else if (((i % N) + (i / N)) % 2 == 0) {
                g.setColor(0xF7F8FA);
                g.fillRoundRect(x + 1, y + 1, s - 2, s - 2, 12, 12);
            }
            boolean here = state == PLAY && i == cy * N + cx;
            boolean hint = hintA >= 0 && (i == hintA || i == hintB);
            if (here) ring(g, x, y, s, picked ? ACCENT : TEXT, picked ? 3 : 2);
            else if (hint) ring(g, x, y, s, ACCENT, 1);
            if (popping && popStage > 0) {
                burst(g, RIMS[board[i]], x + s / 2, y + s / 2, popStage == 1 ? s * 3 / 10 : s / 2, popStage == 2);
                continue;
            }
            int dx = 0, dy = 0;
            if ((phase == SWAP || phase == NOSWAP) && (i == a || i == b)) {
                int o = i == a ? b : a;
                int amt = phase == SWAP ? 100 * (dur - f) / (dur + 1) : WOBBLE[f];
                dx = ((o % N) - (i % N)) * s * amt / 100;
                dy = ((o / N) - (i / N)) * s * amt / 100;
            } else if (phase == FALL && drop[i] > 0) {
                int need = drop[i] * 100;
                dy = -s * Math.max(0, need - travel(f + 1)) / 100;
                if (need > travel(f) && need <= travel(f + 1)) dy = 2; // lands this frame: a small dip
            } else if (hint && motion == 0) {
                dx = frame % 2 == 0 ? -2 : 2;
            }
            if (here && picked) dy -= 2;
            Image[] set = popping ? happy : (here && picked) || hint ? alert : small;
            face(g, board[i], x + s / 2 + dx, y + s / 2 + dy, set);
        }
        for (int i = 0; i < labelAge.length; i++) {
            if (labelAge[i] >= LABEL_F) continue;
            int lx = ox + labelX[i] * s / 100, ly = oy + labelY[i] * s / 100 - lineH / 2 - (motion == 0 ? labelAge[i] * s / 4 : 0);
            int lw = Font.getDefaultFont().stringWidth(labelText[i]) + 10;
            g.setColor(WHITE);
            g.fillRoundRect(lx - lw / 2, ly - 2, lw, lineH + 4, 10, 10);
            g.setColor(ACCENT);
            bold(g, labelText[i], lx, ly, Graphics.TOP | Graphics.HCENTER);
        }
        if (badgeAge < BADGE_F && motion < 2) {
            String t = "연쇄 x" + badgeChain;
            int grow = motion == 0 && badgeAge == 0 ? 4 : 0;
            int bw = Font.getDefaultFont().stringWidth(t) + 24 + 2 * grow, bh = lineH + 12 + 2 * grow;
            int bx = ox + s * N / 2, by = oy + s * N / 2;
            g.setColor(TEXT);
            g.fillRoundRect(bx - bw / 2, by - bh / 2, bw, bh, 18, 18);
            g.setColor(WHITE);
            bold(g, t, bx, by - lineH / 2, Graphics.TOP | Graphics.HCENTER);
        }
    }

    /** A ring and four sparks around (x, y); {@code pale} mixes the colour halfway to white. */
    private static void burst(Graphics g, int rgb, int x, int y, int r, boolean pale) {
        g.setColor(pale ? ((rgb >> 1) & 0x7F7F7F) + 0x808080 : rgb);
        for (int t = 0; t < 2; t++) g.drawRoundRect(x - r + t, y - r + t, 2 * (r - t), 2 * (r - t), 2 * (r - t), 2 * (r - t));
        int d = r * 7 / 10 + 3;
        for (int q = 0; q < 4; q++) g.fillArc(x + (q < 2 ? -d : d) - 2, y + (q % 2 == 0 ? -d : d) - 2, 4, 4, 0, 360);
    }

    private static void ring(Graphics g, int x, int y, int s, int color, int thick) {
        g.setColor(color);
        for (int t = 0; t < thick; t++) g.drawRoundRect(x + t, y + t, s - 1 - 2 * t, s - 1 - 2 * t, 12, 12);
    }

    void paintEmblem(Graphics g, int cx, int cy) {
        int[] trio = {0, 1, 3};
        for (int i = 0; i < 3; i++) face(g, trio[i], cx - 62 + i * 62, cy + ((frame / 2 + i) % 3 == 0 ? -3 : 0), true);
    }

    /** Pastel tiles for the name logo on the title screen, one per letter. */
    private static final int[] TILES = {0xF3D29B, 0xCBE8C9, 0xE4DDF5, 0xCFD7E6, 0xF6D0C4};

    void paintTitle(Graphics g, int w, int bottom) {
        // Two bands of small faces drift past in opposite directions, behind the logo.
        int step = 44, shift = (frame * 4) % step;
        for (int band = 0; band < 2; band++) {
            int y = band == 0 ? 26 : bottom - 26;
            g.setColor(0xEEF0F3);
            g.fillRect(0, y - 20, w, 40);
            for (int i = -1; i <= w / step + 1; i++) {
                int x = band == 0 ? i * step + shift : i * step - shift;
                face(g, ((i + 12) * (band == 0 ? 1 : 5) + band) % 6, x + step / 2, y, false);
            }
        }
        int mid = bottom / 2;
        paintEmblem(g, w / 2, mid - 42);
        String n = name();
        int tile = 40, gap = 4, x0 = (w - n.length() * (tile + gap) + gap) / 2;
        for (int i = 0; i < n.length(); i++) {
            int x = x0 + i * (tile + gap), y = mid - 4 - ((frame + i) % 5 == 0 ? 3 : 0);
            g.setColor(0xD8DCE2);
            g.fillRoundRect(x, y + 2, tile, tile, 14, 14);
            g.setColor(TILES[i % TILES.length]);
            g.fillRoundRect(x, y, tile, tile, 14, 14);
            g.setColor(TEXT);
            bold(g, n.substring(i, i + 1), x + tile / 2, y + (tile - lineH) / 2, Graphics.TOP | Graphics.HCENTER);
        }
        g.setColor(SUB);
        g.drawString(tagline(), w / 2, mid + tile + 6, Graphics.TOP | Graphics.HCENTER);
    }

    String[] helpTopics() {
        return new String[] {"기본 규칙", "점수 계산", "연쇄", "힌트와 섞기", "난이도", "동물 친구들", "조작 키", "설정"};
    }

    private static final String[] ANIMALS = {"곰", "수달", "고양이", "토끼", "펭귄", "개구리"};
    private static final String[] EARS = {"둥근 귀", "작은 옆 귀", "뾰족한 귀", "긴 귀", "흰 얼굴 가면", "머리 위 눈"};

    void paintHelp(Graphics g, int page, int top, int w, int bottom) {
        int cx = w / 2;
        int y = top + 6;
        int cap = bottom - 2 * lineH - 6; // two caption lines at the foot of the page
        String[] lines;
        if (page == 0) {
            int[] pic = {0, 0, 2, 3, 4, 0};
            int s = 56, ox = cx - 3 * s / 2;
            card(g, ox - 4, y - 4, 3 * s + 8, 2 * s + 8, 18);
            for (int i = 0; i < 6; i++) face(g, pic[i], ox + (i % 3) * s + s / 2, y + (i / 3) * s + s / 2, true);
            ring(g, ox + 2 * s, y, s, ACCENT, 3);
            ring(g, ox + 2 * s, y + s, s, ACCENT, 1);
            arrow(g, ox + 2 * s + s / 2, y + s - 6);
            g.setColor(TEXT);
            bold(g, "옆 동물과 자리를 바꿔요", cx, y + 2 * s + 16, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"같은 동물 셋 이상이 한 줄이면", "사라지고 점수를 얻어요"};
        } else if (page == 1) {
            for (int r = 0; r < 3; r++) {
                int n = 3 + r, ry = y + r * 38, ox = 22;
                if (r == 0) {
                    g.setColor(0xFFF1BF);
                    g.fillRoundRect(ox - 4, ry - 2, n * 32 + 8, 34, 14, 14);
                }
                for (int i = 0; i < n; i++) face(g, r == 1 ? 3 : r == 2 ? 5 : 0, ox + i * 32 + 16, ry + 15, false);
                g.setColor(ACCENT);
                bold(g, "+" + n * 10, w - 22, ry + 15 - lineH / 2, Graphics.TOP | Graphics.RIGHT);
            }
            int by = y + 3 * 38 + 6;
            g.setColor(TEXT);
            g.fillRoundRect(cx - 100, by, 200, lineH + 10, 16, 16);
            g.setColor(WHITE);
            bold(g, "연쇄 둘째 x2, 셋째 x3", cx, by + 5, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"동물 하나에 10점이에요", "연쇄마다 점수가 곱절로 커져요"};
        } else if (page == 2) {
            int s = 52;
            int[] col = {2, 5, 2};
            for (int i = 0; i < 3; i++) face(g, col[i], cx - 50, y + i * (s - 6) + s / 2, true);
            arrow(g, cx - 50, y + 2);
            g.setColor(TEXT);
            g.fillRoundRect(cx, y + s, 90, lineH + 10, 16, 16);
            g.setColor(WHITE);
            bold(g, "연쇄 x2", cx + 45, y + s + 5, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"빈자리로 떨어져 또 맞으면", "연쇄예요. 점수가 커져요"};
        } else if (page == 3) {
            int[] pic = {1, 3, 0, 4, 2, 2, 5, 0, 3};
            int s = 40, ox = cx - 3 * s / 2;
            card(g, ox - 4, y - 4, 3 * s + 8, 3 * s + 8, 16);
            for (int i = 0; i < 9; i++) {
                int x = ox + (i % 3) * s, yy = y + (i / 3) * s;
                boolean hint = i == 3 || i == 4;
                if (hint) ring(g, x, yy, s, ACCENT, 2);
                face(g, pic[i], x + s / 2, yy + s / 2, hint ? alert : small);
            }
            g.setColor(TEXT);
            g.fillRoundRect(cx - 70, y + 3 * s + 12, 140, lineH + 8, 14, 14);
            g.setColor(WHITE);
            g.drawString("판을 새로 섞었어요", cx, y + 3 * s + 16, Graphics.TOP | Graphics.HCENTER);
            lines = new String[] {"잠시 멈추면 둘 곳이 반짝여요", "둘 곳이 없으면 판을 섞어요"};
        } else if (page == 4) {
            for (int l = 0; l < 2; l++) {
                int ry = y + l * 70, n = KINDS[l];
                card(g, 14, ry, w - 28, 62, 16);
                g.setColor(TEXT);
                bold(g, LEVELS[l], 26, ry + 8, Graphics.TOP | Graphics.LEFT);
                g.setColor(SUB);
                g.drawString(l == 0 ? "동물 다섯" : "동물 여섯", w - 26, ry + 8, Graphics.TOP | Graphics.RIGHT);
                for (int k = 0; k < n; k++) face(g, k, 36 + k * 28, ry + 40, false);
                g.setColor(SURFACE2);
                g.fillRoundRect(200 - 30, ry + 36, 44, 7, 7, 7);
                g.setColor(0x4CC38A);
                g.fillRoundRect(200 - 30, ry + 36, l == 0 ? 44 : 30, 7, 7, 7);
            }
            lines = new String[] {"위 막대가 남은 시간이에요", "기록은 난이도마다 따로 남아요"};
        } else if (page == 5) {
            for (int k = 0; k < 6; k++) {
                int ry = y + k * 30;
                face(g, k, 36, ry + 13, false);
                g.setColor(TEXT);
                bold(g, ANIMALS[k], 60, ry + 13 - lineH / 2, Graphics.TOP | Graphics.LEFT);
                g.setColor(SUB);
                g.drawString(EARS[k], w - 18, ry + 13 - lineH / 2, Graphics.TOP | Graphics.RIGHT);
            }
            lines = new String[] {"", "귀 모양으로도 알아볼 수 있어요"};
        } else if (page == 6) {
            String[][] keys = {{"2 4 6 8", "움직이기"}, {"5", "집기, 놓기"}, {"L", "고르기"}, {"R CLR", "뒤로 가기"}, {"종료", "게임 나가기"}};
            for (int i = 0; i < keys.length; i++) {
                int ky = y + i * (lineH + 12);
                g.setColor(i == 4 ? 0xE8685A : TEXT);
                g.fillRoundRect(18, ky - 4, 66, lineH + 8, 12, 12);
                g.setColor(WHITE);
                bold(g, keys[i][0], 51, ky, Graphics.TOP | Graphics.HCENTER);
                g.setColor(TEXT);
                g.drawString(keys[i][1], 96, ky, Graphics.TOP | Graphics.LEFT);
            }
            lines = new String[] {"게임 중 L R CLR 은", "메뉴를 열고 시간을 멈춰요"};
        } else {
            String[][] rows = {{"소리", "효과음을 켜고 꺼요"}, {"진동", "터질 때 떨려요"}, {"난이도", "쉬움, 보통"}, {"움직임", "켬, 줄임, 끔"}};
            for (int i = 0; i < rows.length; i++) {
                int ry = y + i * (lineH + 20);
                card(g, 14, ry, w - 28, lineH + 12, 12);
                g.setColor(TEXT);
                bold(g, rows[i][0], 26, ry + 6, Graphics.TOP | Graphics.LEFT);
                g.setColor(SUB);
                g.drawString(rows[i][1], w - 26, ry + 6, Graphics.TOP | Graphics.RIGHT);
            }
            lines = new String[] {"움직임 끔이면 판이 바로 바뀌어요", "메뉴에서 언제든 바꿀 수 있어요"};
        }
        g.setColor(SUB);
        for (int i = 0; i < lines.length; i++) g.drawString(lines[i], cx, cap + i * (lineH + 4), Graphics.TOP | Graphics.HCENTER);
    }

    /** A small down-pointing triangle under (x, y). */
    private static void arrow(Graphics g, int x, int y) {
        g.setColor(ACCENT);
        for (int r = 0; r < 7; r++) g.drawLine(x - 7 + r, y + r, x + 7 - r, y + r);
    }
}
