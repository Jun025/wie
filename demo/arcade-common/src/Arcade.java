/*
 * otterpebble arcade — Copyright (c) 2026 otterpebble. MIT License (see LICENSE).
 *
 * The frame a demo game sits in: its own home screen (always first), settings, a picture
 * help, the best scores, the play loop with a shrinking time bar, and the "quit?" card.
 * CLR is "back" everywhere: from settings, help and the scores it goes home at once;
 * during play it pauses and asks first; at home it does nothing (leaving the game is the
 * shell's job). A game only fills in the rules, the board and the help pictures.
 * Colours follow otterpebble DESIGN.md: a light neutral surface, one text colour, and
 * colour only where it means something.
 */

import java.util.Random;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;

abstract class Arcade extends Canvas implements Runnable {
    static final int HOME = 0, PLAY = 1, OVER = 2, SETTINGS = 3, HELP = 4, SCORES = 5, QUIT = 6, RESET = 7;
    /**
     * CLR as it reaches {@code keyPressed} in wie — measured, not assumed: the host's CLEAR key
     * becomes MIDP key code 8 (wie-midp {@code MIDPKeyCode::CLEAR}), with game action 0. Some
     * handsets send -8; both mean back here.
     */
    static final int KEY_CLR = 8;

    // DESIGN.md light tokens: --background, --surface, --surface-2, --text, --text-sub.
    static final int WHITE = 0xFFFFFF;
    static final int SURFACE = 0xF4F5F7;
    static final int SURFACE2 = 0xEBEDF0;
    static final int TEXT = 0x191F28;
    static final int SUB = 0x616B78;
    static final int ACCENT = 0x3182F6;

    static final String[] MENU = {"시작", "설정", "도움말", "최고 점수", "기록 초기화"};
    static final String[] LEVELS = {"쉬움", "보통"};
    /** The motion setting: full animation, a shortened one, or none (the board just changes). */
    static final String[] MOTIONS = {"켬", "줄임", "끔"};
    static final String[] MOTION_NOTES = {"움직임을 다 보여 줘요", "짧게, 흔들림 없이", "움직임 없이 바로"};

    final Sound sound = new Sound();
    final Random random = new Random();
    final int lineH;
    Display display;

    volatile boolean running = true;
    int state = HOME;
    int menu, row, page;
    int score;
    boolean newBest;
    int frame;
    long overAt;
    int overFrame;
    String toast;
    long toastUntil;

    // Saved in one record: best score per level, then sound · vibration · level · motion.
    final int[] best = new int[2];
    boolean vibrate = true;
    int level = 1;
    int motion;

    // The round clock: milliseconds left, counted down only while playing, so the quit card pauses it.
    int roundMs, leftMs;
    private long lastTick;

    Arcade() {
        setFullScreenMode(true);
        lineH = Font.getDefaultFont().getHeight();
        load();
    }

    // ---- what a game supplies ----
    abstract String name();

    abstract String tagline();

    /** A new round at {@code level}; returns its length in milliseconds. */
    abstract int newGame(int level);

    /** A key during play. {@code action} is the game action with 2/4/6/8/5 folded in. */
    abstract void key(int action);

    /** Called every loop turn while playing. */
    void tick() {}

    /** The time ran out: finish whatever is still moving, so a chain in flight still scores. */
    void end() {}

    abstract void paintBoard(Graphics g, int top, int w, int h);

    /** The picture above the name on the home screen, centred on (cx, cy). */
    abstract void paintEmblem(Graphics g, int cx, int cy);

    abstract int helpPages();

    /** One help page between {@code top} and {@code bottom}; returns its title. */
    abstract String paintHelp(Graphics g, int page, int top, int w, int bottom);

    // ---- helpers for games ----
    int rnd(int n) {
        return (random.nextInt() >>> 1) % n;
    }

    void buzz(int ms) {
        if (vibrate && display != null) display.vibrate(ms);
    }

    void toast(String text) {
        toast = text;
        toastUntil = System.currentTimeMillis() + 1200;
    }

    /** The host font has one size, so emphasis is weight: the string drawn twice, 1px apart. */
    static void bold(Graphics g, String s, int x, int y, int anchor) {
        g.drawString(s, x, y, anchor);
        g.drawString(s, x + 1, y, anchor);
    }

    /** A rounded card with a 1px rim, so it separates from the surface without a shadow. */
    static void card(Graphics g, int x, int y, int w, int h, int r) {
        g.setColor(SURFACE2);
        g.fillRoundRect(x - 1, y - 1, w + 2, h + 2, r + 2, r + 2);
        g.setColor(WHITE);
        g.fillRoundRect(x, y, w, h, r, r);
    }

    private void gameOver() {
        end();
        state = OVER;
        overAt = System.currentTimeMillis();
        overFrame = frame;
        newBest = score > best[level];
        if (newBest) {
            best[level] = score;
            save();
        }
        sound.play(Sound.OVER);
        buzz(200);
    }

    private void start() {
        score = 0;
        roundMs = leftMs = newGame(level);
        lastTick = System.currentTimeMillis();
        state = PLAY;
        sound.play(Sound.START);
    }

    // ---- the frame ----
    public void run() {
        while (running) {
            synchronized (this) {
                if (state == PLAY) {
                    long now = System.currentTimeMillis();
                    leftMs -= (int) (now - lastTick);
                    lastTick = now;
                    tick();
                    if (leftMs <= 0) {
                        leftMs = 0;
                        gameOver();
                    }
                }
            }
            frame++;
            repaint();
            try {
                Thread.sleep(state == PLAY ? 100 : 200);
            } catch (InterruptedException e) {
                return;
            }
        }
    }

    void quit() {
        running = false;
        sound.stop();
    }

    protected synchronized void keyPressed(int key) {
        boolean back = key == KEY_CLR || key == -KEY_CLR;
        int action = back ? 0 : getGameAction(key);
        // Keypad digits first: handsets differ in which digits they report as game actions.
        if (key == KEY_NUM2) action = UP;
        else if (key == KEY_NUM8) action = DOWN;
        else if (key == KEY_NUM4) action = LEFT;
        else if (key == KEY_NUM6) action = RIGHT;
        else if (key == KEY_NUM5) action = FIRE;

        switch (state) {
            case HOME:
                if (action == UP || action == DOWN) {
                    menu = (menu + (action == UP ? MENU.length - 1 : 1)) % MENU.length;
                    sound.play(Sound.MOVE);
                } else if (action == FIRE) {
                    if (menu == 0) start();
                    else if (menu == 1) state = SETTINGS;
                    else if (menu == 2) state = HELP;
                    else if (menu == 3) state = SCORES;
                    else state = RESET;
                    row = page = 0;
                }
                break;
            case PLAY:
                if (back) state = QUIT;
                else key(action);
                break;
            case QUIT:
                if (action == FIRE) state = HOME;
                else if (back) {
                    state = PLAY;
                    lastTick = System.currentTimeMillis();
                }
                break;
            case RESET:
                if (action == FIRE) {
                    best[0] = best[1] = 0;
                    save();
                    toast("기록을 지웠어요");
                }
                if (action == FIRE || back) state = HOME;
                break;
            case OVER:
                // A short lock so the key that ended the round does not also start the next one.
                if (System.currentTimeMillis() - overAt < 800) break;
                if (action == FIRE) start();
                else if (back) state = HOME;
                break;
            case SETTINGS:
                if (back) {
                    save();
                    state = HOME;
                } else if (action == UP || action == DOWN) {
                    row = (row + (action == UP ? 3 : 1)) % 4;
                } else if (action == FIRE || action == LEFT || action == RIGHT) {
                    if (row == 0) {
                        sound.enabled = !sound.enabled;
                        if (!sound.enabled) sound.stop();
                    } else if (row == 1) {
                        vibrate = !vibrate;
                        buzz(80);
                    } else if (row == 2) {
                        level = 1 - level;
                    } else {
                        motion = (motion + (action == LEFT ? 2 : 1)) % 3;
                    }
                    sound.play(Sound.TOGGLE);
                }
                break;
            case HELP:
                if (back || (action == FIRE && page == helpPages() - 1)) state = HOME;
                else if ((action == RIGHT || action == FIRE) && page < helpPages() - 1) page++;
                else if (action == LEFT && page > 0) page--;
                break;
            default: // SCORES
                if (back || action == FIRE) state = HOME;
        }
        repaint();
    }

    protected void paint(Graphics g) {
        int w = getWidth();
        int h = getHeight();
        g.setColor(SURFACE);
        g.fillRect(0, 0, w, h);
        synchronized (this) {
            if (state == HOME || state == RESET) {
                paintHome(g, w, h);
                if (state == RESET) ask(g, w, h, "최고 점수를 지울까요?");
            } else if (state == SETTINGS) {
                paintSettings(g, w, h);
            } else if (state == HELP) {
                paintHelpPage(g, w, h);
            } else if (state == SCORES) {
                paintScores(g, w, h);
            } else {
                paintPlay(g, w, h);
                if (state == QUIT) ask(g, w, h, "그만할까요?");
                if (state == OVER) paintOver(g, w, h);
            }
        }
        if (toast != null && System.currentTimeMillis() < toastUntil) {
            int tw = Font.getDefaultFont().stringWidth(toast) + 20;
            g.setColor(TEXT);
            g.fillRoundRect((w - tw) / 2, h - lineH - 22, tw, lineH + 10, 12, 12);
            g.setColor(WHITE);
            g.drawString(toast, w / 2, h - lineH - 17, Graphics.TOP | Graphics.HCENTER);
        }
    }

    /** A title bar with a "CLR 뒤로" hint, for the screens CLR leaves at once. */
    private int header(Graphics g, int w, String title) {
        int bar = lineH + 12;
        g.setColor(WHITE);
        g.fillRect(0, 0, w, bar);
        g.setColor(TEXT);
        bold(g, title, w / 2, 6, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("CLR", 8, 6, Graphics.TOP | Graphics.LEFT);
        return bar;
    }

    private void paintHome(Graphics g, int w, int h) {
        int cx = w / 2;
        paintEmblem(g, cx, 52);
        int y = 92;
        g.setColor(TEXT);
        bold(g, name(), cx, y, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString(tagline(), cx, y + lineH + 4, Graphics.TOP | Graphics.HCENTER);

        int step = lineH + 12;
        y += 2 * lineH + 18;
        for (int i = 0; i < MENU.length; i++) {
            int by = y + i * step;
            if (i == menu) {
                g.setColor(TEXT);
                g.fillRoundRect(cx - 70, by, 140, lineH + 8, 18, 18);
                g.setColor(WHITE);
                bold(g, MENU[i], cx, by + 4, Graphics.TOP | Graphics.HCENTER);
            } else {
                card(g, cx - 70, by, 140, lineH + 8, 18);
                g.setColor(TEXT);
                g.drawString(MENU[i], cx, by + 4, Graphics.TOP | Graphics.HCENTER);
            }
        }
        g.setColor(SUB);
        g.drawString("2 8 고르기 · 5 선택", cx, h - 2 * lineH - 10, Graphics.TOP | Graphics.HCENTER);
        g.drawString("otterpebble", cx, h - lineH - 6, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintSettings(Graphics g, int w, int h) {
        int y = header(g, w, "설정") + 16;
        String[] names = {"소리", "진동", "난이도", "움직임"};
        String[] values = {sound.enabled ? "켬" : "끔", vibrate ? "켬" : "끔", LEVELS[level], MOTIONS[motion]};
        int rh = lineH + 16;
        for (int i = 0; i < 4; i++) {
            int ry = y + i * (rh + 8);
            if (i == row) {
                g.setColor(TEXT);
                g.fillRoundRect(14, ry - 2, w - 28, rh + 4, 16, 16);
            }
            card(g, 16, ry, w - 32, rh, 14);
            g.setColor(TEXT);
            g.drawString(names[i], 30, ry + 8, Graphics.TOP | Graphics.LEFT);
            g.setColor(i == 2 || values[i].equals("켬") ? ACCENT : SUB);
            bold(g, values[i], w - 30, ry + 8, Graphics.TOP | Graphics.RIGHT);
        }
        y += 4 * (rh + 8) + 6;
        g.setColor(SUB);
        String note = row == 3 ? MOTION_NOTES[motion] : level == 0 ? "쉬움: 동물 다섯 · 시간 넉넉히" : "보통: 동물 여섯 · 시간 보통";
        g.drawString(note, w / 2, y, Graphics.TOP | Graphics.HCENTER);
        g.drawString("2 8 고르기 · 5 바꾸기", w / 2, h - 2 * lineH - 10, Graphics.TOP | Graphics.HCENTER);
        g.drawString("CLR 저장하고 처음으로", w / 2, h - lineH - 6, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintHelpPage(Graphics g, int w, int h) {
        int top = header(g, w, "도움말 " + (page + 1) + "/" + helpPages());
        int bottom = h - 2 * lineH - 16;
        String title = paintHelp(g, page, top + 8, w, bottom);
        g.setColor(TEXT);
        bold(g, title, w / 2, top + 10, Graphics.TOP | Graphics.HCENTER);
        for (int i = 0; i < helpPages(); i++) {
            g.setColor(i == page ? TEXT : SURFACE2);
            g.fillArc(w / 2 - helpPages() * 7 + i * 14 + 2, bottom + 4, 8, 8, 0, 360);
        }
        g.setColor(SUB);
        g.drawString(page < helpPages() - 1 ? "4 6 넘기기 · 5 다음" : "4 앞으로 · 5 처음으로", w / 2, h - lineH - 6,
            Graphics.TOP | Graphics.HCENTER);
    }

    private void paintScores(Graphics g, int w, int h) {
        int y = header(g, w, "최고 점수") + 24;
        for (int i = 0; i < 2; i++) {
            card(g, 24, y, w - 48, 2 * lineH + 22, 16);
            g.setColor(SUB);
            g.drawString(LEVELS[i], w / 2, y + 8, Graphics.TOP | Graphics.HCENTER);
            g.setColor(TEXT);
            bold(g, best[i] + " 점", w / 2, y + lineH + 12, Graphics.TOP | Graphics.HCENTER);
            y += 2 * lineH + 34;
        }
        g.setColor(SUB);
        g.drawString("5 또는 CLR 처음으로", w / 2, h - lineH - 6, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintPlay(Graphics g, int w, int h) {
        int bar = lineH + 10;
        int top = bar + 10;
        paintBoard(g, top, w, h);
        g.setColor(WHITE);
        g.fillRect(0, 0, w, top);
        g.setColor(TEXT);
        bold(g, "점수 " + score, 8, 5, Graphics.TOP | Graphics.LEFT);
        g.setColor(SUB);
        g.drawString("최고 " + best[level], w - 8, 5, Graphics.TOP | Graphics.RIGHT);

        // The time bar: no seconds anywhere, only how much is left. Green, then yellow under
        // a half, red under a fifth, and it blinks in the last tenth.
        int tw = w - 16;
        g.setColor(SURFACE2);
        g.fillRoundRect(8, bar, tw, 7, 7, 7);
        int fill = roundMs == 0 ? 0 : (int) ((long) tw * leftMs / roundMs);
        boolean blink = leftMs * 10 < roundMs && state == PLAY && (frame / 2) % 2 == 1;
        if (fill > 0 && !blink) {
            g.setColor(leftMs * 5 < roundMs ? 0xE8685A : leftMs * 2 < roundMs ? 0xF2B84B : 0x4CC38A);
            g.fillRoundRect(8, bar, Math.max(fill, 7), 7, 7, 7);
        }
    }

    /** How many frames the end card's score takes to count up. */
    static final int COUNT_FRAMES = 6;

    /** The yes/no card: 5 = yes, CLR = no. */
    private void ask(Graphics g, int w, int h, String question) {
        int cw = w - 48;
        int ch = 3 * lineH + 40;
        int x = 24, y = (h - ch) / 2;
        g.setColor(TEXT);
        g.fillRoundRect(x - 2, y - 2, cw + 4, ch + 4, 22, 22);
        g.setColor(WHITE);
        g.fillRoundRect(x, y, cw, ch, 20, 20);
        g.setColor(TEXT);
        bold(g, question, w / 2, y + 14, Graphics.TOP | Graphics.HCENTER);
        int by = y + 2 * lineH + 12;
        g.fillRoundRect(w / 2 - 94, by - 4, 88, lineH + 8, 16, 16);
        g.setColor(WHITE);
        g.drawString("5 예", w / 2 - 50, by, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SURFACE2);
        g.fillRoundRect(w / 2 + 6, by - 4, 88, lineH + 8, 16, 16);
        g.setColor(TEXT);
        g.drawString("CLR 아니오", w / 2 + 50, by, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintOver(Graphics g, int w, int h) {
        int cw = w - 48;
        int ch = 5 * lineH + 56;
        int x = 24, y = (h - ch) / 2;
        g.setColor(TEXT);
        g.fillRoundRect(x - 2, y - 2, cw + 4, ch + 4, 22, 22);
        g.setColor(WHITE);
        g.fillRoundRect(x, y, cw, ch, 20, 20);
        int cx = w / 2;
        g.setColor(TEXT);
        bold(g, "시간 끝", cx, y + 12, Graphics.TOP | Graphics.HCENTER);
        // The score counts up over a few frames; a new best then blinks. Motion off shows it at once.
        int age = frame - overFrame;
        boolean counting = motion == 0 && age < COUNT_FRAMES;
        bold(g, (counting ? score * age / COUNT_FRAMES : score) + " 점", cx, y + lineH + 20, Graphics.TOP | Graphics.HCENTER);
        if (!counting && !(newBest && motion == 0 && (age / 2) % 2 == 1)) {
            g.setColor(newBest ? ACCENT : SUB);
            g.drawString(newBest ? "새 최고 기록!" : "최고 " + best[level], cx, y + 2 * lineH + 26, Graphics.TOP | Graphics.HCENTER);
        }
        g.setColor(SURFACE2);
        g.fillRoundRect(cx - 60, y + 3 * lineH + 34, 120, lineH + 8, 16, 16);
        g.setColor(TEXT);
        g.drawString("5 다시하기", cx, y + 3 * lineH + 38, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("CLR 처음으로", cx, y + 4 * lineH + 44, Graphics.TOP | Graphics.HCENTER);
    }

    // ---- one record, named after the game: best easy · best normal (4 bytes each), sound · vibe · level · motion.
    // An 11-byte record (from before motion existed) still loads; motion then stays on. ----
    private void load() {
        try {
            RecordStore rs = RecordStore.openRecordStore(name(), true);
            if (rs.getNumRecords() > 0) {
                byte[] b = rs.getRecord(1);
                if (b != null && b.length >= 11) {
                    for (int i = 0; i < 2; i++) {
                        best[i] = ((b[4 * i] & 0xFF) << 24) | ((b[4 * i + 1] & 0xFF) << 16) | ((b[4 * i + 2] & 0xFF) << 8) | (b[4 * i + 3] & 0xFF);
                    }
                    sound.enabled = b[8] != 0;
                    vibrate = b[9] != 0;
                    level = b[10] == 0 ? 0 : 1;
                    if (b.length >= 12 && b[11] >= 0 && b[11] < 3) motion = b[11];
                }
            }
            rs.closeRecordStore();
        } catch (Exception e) {
            // no storage: defaults, and nothing lasts past closing the game
        }
    }

    private void save() {
        byte[] b = new byte[12];
        for (int i = 0; i < 2; i++) {
            int v = best[i];
            b[4 * i] = (byte) (v >>> 24);
            b[4 * i + 1] = (byte) (v >>> 16);
            b[4 * i + 2] = (byte) (v >>> 8);
            b[4 * i + 3] = (byte) v;
        }
        b[8] = (byte) (sound.enabled ? 1 : 0);
        b[9] = (byte) (vibrate ? 1 : 0);
        b[10] = (byte) level;
        b[11] = (byte) motion;
        try {
            RecordStore rs = RecordStore.openRecordStore(name(), true);
            if (rs.getNumRecords() > 0) rs.setRecord(1, b, 0, b.length);
            else rs.addRecord(b, 0, b.length);
            rs.closeRecordStore();
        } catch (Exception e) {
            // keep playing without saving
        }
    }

    /** Every game's MIDlet: show the canvas, start its loop. */
    abstract static class Midlet extends MIDlet {
        private Arcade canvas;

        abstract Arcade create();

        protected void startApp() {
            if (canvas == null) {
                canvas = create();
                canvas.display = Display.getDisplay(this);
                canvas.display.setCurrent(canvas);
                new Thread(canvas).start();
            }
        }

        protected void pauseApp() {}

        protected void destroyApp(boolean unconditional) {
            if (canvas != null) canvas.quit();
        }
    }
}
