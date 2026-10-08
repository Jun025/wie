/*
 * otterpebble arcade — Copyright (c) 2026 otterpebble. MIT License (see LICENSE).
 *
 * The frame a demo game sits in, laid out like a feature-phone game of the era:
 * a short maker logo, the title screen ("press any key"), a main menu (start with a level pick,
 * continue, how to play, settings, best scores, credits, quit), the play loop with a
 * shrinking time bar, a pause menu, and yes/no cards. Every screen shows its soft-key labels
 * along the bottom. A game only fills in the rules, the board, the title picture and the help.
 *
 * Keys, as they reach keyPressed in wie: the standard J2ME codes since wie gave them to J2ME
 * (2026-10-09) — left soft key -6, right soft key -7, CLR -8, the red end key (HANGUP) -11,
 * up -1. An older wie sent the SKT values (6, 7, 8, end key -1 — measured with a probe MIDlet,
 * docs/report/0379); both are accepted. Left soft = select, right soft and CLR = back, and in
 * play all three open the pause menu. The end key asks to quit from anywhere.
 * Colours follow otterpebble DESIGN.md: a light neutral surface, one text colour, and
 * colour only where it means something. Screen text: no dashes, no middle dots, 해요체.
 */

import java.util.Random;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Font;
import javax.microedition.lcdui.Graphics;
import javax.microedition.midlet.MIDlet;
import javax.microedition.rms.RecordStore;

abstract class Arcade extends Canvas implements Runnable {
    static final int LOGO = 0, TITLE = 1, MENU = 2, LEVEL = 3, PLAY = 4, PAUSE = 5, OVER = 6, SETTINGS = 7, HELP = 8, PAGE = 9,
        SCORES = 10, CREDITS = 11, ASK = 12;
    /** The older wie sent these positive; both signs are accepted. */
    static final int KEY_LSOFT = 6, KEY_RSOFT = 7, KEY_CLR = 8, KEY_HANGUP = -11;

    // DESIGN.md light tokens: --background, --surface, --surface-2, --text, --text-sub.
    static final int WHITE = 0xFFFFFF;
    static final int SURFACE = 0xF4F5F7;
    static final int SURFACE2 = 0xEBEDF0;
    static final int TEXT = 0x191F28;
    static final int SUB = 0x616B78;
    static final int ACCENT = 0x3182F6;

    /** Main menu entries; 이어하기 is only listed while a paused round is kept. */
    static final int GO = 0, RESUME = 1, HOWTO = 2, SETUP = 3, BEST = 4, ABOUT = 5, EXIT = 6, HOME = 7;
    static final String[] MAIN = {"게임 시작", "이어하기", "게임 방법", "설정", "최고 기록", "제작자 정보", "게임 나가기"};
    static final int[] PAUSED = {RESUME, HOWTO, SETUP, HOME, EXIT};
    static final String[] PAUSED_LABELS = {"계속하기", "게임 방법", "설정", "처음 화면으로", "게임 나가기"};
    /** One pastel per icon, so the list reads by colour as well as by word. */
    static final int[] ICON_BG = {0xCBE8C9, 0xCFE0F7, 0xF3D29B, 0xE4DDF5, 0xF6E2A0, 0xD4ECEE, 0xF6D0C4, 0xEDE0C8};

    static final String[] LEVELS = {"쉬움", "보통"};
    static final String[] LEVEL_NOTES = {"동물 다섯, 시간 넉넉히", "동물 여섯, 시간 보통"};
    /** The motion setting: full animation, a shortened one, or none (the board just changes). */
    static final String[] MOTIONS = {"켬", "줄임", "끔"};
    static final String[] MOTION_NOTES = {"움직임을 다 보여 줘요", "짧게, 흔들림 없이 보여 줘요", "움직임 없이 바로 바뀌어요"};

    static final int ASK_EXIT = 0, ASK_HOME = 1, ASK_RESET = 2;

    /** Loop turns the maker logo stays up: the first launch, then every launch after. */
    static final int LOGO_FIRST = 9, LOGO_AGAIN = 3;

    final Sound sound = new Sound();
    final Random random = new Random();
    final int lineH;
    Display display;
    MIDlet midlet;

    volatile boolean running = true;
    int state = LOGO;
    /** Where help and settings go back to: the main menu or the pause menu. */
    int from = MENU;
    int menu, row, page, topic;
    int[] items = new int[0];
    int ask, askBack;
    int logoLeft;
    int score;
    boolean newBest;
    int frame;
    long overAt;
    int overFrame;
    String toast;
    long toastUntil;

    // Saved in one record: best score per level, then sound · vibration · level · motion, whether
    // the logo was seen, then the paused round (if any).
    final int[] best = new int[2];
    boolean vibrate = true;
    int level = 1;
    int motion;
    boolean seen;
    /** The paused round: level, score, time left, round length, then the game's own bytes. */
    byte[] resume;

    /** The level of the round in play; the setting may change under a paused round, this does not. */
    int playLevel;
    // The round clock: milliseconds left, counted down only while playing, so every menu pauses it.
    int roundMs, leftMs;
    private long lastTick;

    Arcade() {
        setFullScreenMode(true);
        lineH = Font.getDefaultFont().getHeight();
        load();
        logoLeft = seen ? LOGO_AGAIN : LOGO_FIRST;
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

    /** Finish whatever is still moving (the time ran out, or the round is being paused), so a chain in flight still scores. */
    void end() {}

    /** The board as bytes, for 이어하기; {@link #restore} reads it back. */
    abstract byte[] snapshot();

    /** Puts back a board from {@link #snapshot}; false if the bytes do not make a board. */
    abstract boolean restore(byte[] b, int off, int len);

    abstract void paintBoard(Graphics g, int top, int w, int h);

    /** The picture above the name on the menu and credits, centred on (cx, cy). */
    abstract void paintEmblem(Graphics g, int cx, int cy);

    /** The title screen's picture: the game's name as a logo over a moving backdrop, above {@code bottom}. */
    abstract void paintTitle(Graphics g, int w, int bottom);

    /** The help topics, in order; a topic is one page. */
    abstract String[] helpTopics();

    /** One help page between {@code top} and {@code bottom}. */
    abstract void paintHelp(Graphics g, int page, int top, int w, int bottom);

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

    /** A small triangle pointing right, its left edge at x, centred on y. */
    static void play(Graphics g, int x, int y, int r) {
        for (int i = 0; i <= r; i++) g.drawLine(x + i, y - r + i, x + i, y + r - i);
    }

    // ---- the flow ----
    private void gameOver() {
        end();
        state = OVER;
        overAt = System.currentTimeMillis();
        overFrame = frame;
        newBest = score > best[playLevel];
        if (newBest) best[playLevel] = score;
        resume = null;
        save();
        sound.play(Sound.OVER);
        buzz(200);
    }

    private void start() {
        score = 0;
        playLevel = level;
        roundMs = leftMs = newGame(level);
        lastTick = System.currentTimeMillis();
        resume = null;
        save();
        state = PLAY;
        sound.play(Sound.START);
    }

    /** Stops the clock, keeps the round for 이어하기, and opens the pause menu. */
    private void pause() {
        end();
        byte[] game = snapshot();
        byte[] r = new byte[13 + game.length];
        r[0] = (byte) playLevel;
        putInt(r, 1, score);
        putInt(r, 5, leftMs);
        putInt(r, 9, roundMs);
        System.arraycopy(game, 0, r, 13, game.length);
        resume = r;
        save();
        state = PAUSE;
        menu = 0;
    }

    private void unpause() {
        state = PLAY;
        lastTick = System.currentTimeMillis();
    }

    private void resumeGame() {
        byte[] r = resume;
        if (r == null || r.length < 13 || !restore(r, 13, r.length - 13)) {
            resume = null;
            save();
            toMenu(GO);
            toast("이어할 판이 없어요");
            return;
        }
        playLevel = r[0] == 0 ? 0 : 1;
        score = getInt(r, 1);
        leftMs = getInt(r, 5);
        roundMs = getInt(r, 9);
        unpause();
        sound.play(Sound.START);
    }

    private void toMenu(int select) {
        state = MENU;
        items = new int[resume == null ? MAIN.length - 1 : MAIN.length];
        int n = 0;
        for (int i = 0; i < MAIN.length; i++) if (i != RESUME || resume != null) items[n++] = i;
        menu = 0;
        for (int i = 0; i < items.length; i++) if (items[i] == select) menu = i;
    }

    private void toTitle() {
        state = TITLE;
        if (!seen) {
            seen = true;
            save();
        }
    }

    private void askFor(int kind) {
        ask = kind;
        askBack = state;
        state = ASK;
    }

    /** Leaves the game: the host closes it (and shows its own "the game ended" panel). */
    private void exit() {
        save();
        quit();
        if (midlet != null) midlet.notifyDestroyed();
    }

    /** Opens a main-menu or pause-menu entry. */
    private void choose(int item) {
        row = page = topic = 0;
        if (item == GO) {
            state = LEVEL;
            row = level;
        } else if (item == RESUME) {
            if (state == PAUSE) unpause();
            else resumeGame();
        } else if (item == HOWTO) {
            from = state;
            state = HELP;
        } else if (item == SETUP) {
            from = state;
            state = SETTINGS;
        } else if (item == BEST) {
            state = SCORES;
        } else if (item == ABOUT) {
            state = CREDITS;
        } else if (item == HOME) {
            askFor(ASK_HOME);
        } else {
            askFor(ASK_EXIT);
        }
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
                } else if (state == LOGO && --logoLeft <= 0) {
                    toTitle();
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
        boolean lsoft = key == KEY_LSOFT || key == -KEY_LSOFT;
        boolean back = key == KEY_RSOFT || key == -KEY_RSOFT || key == KEY_CLR || key == -KEY_CLR;
        // -1 is up under the standard codes; only an older wie (up = 141) meant the end key by it.
        boolean hangup = key == KEY_HANGUP || key == -1 && getKeyCode(UP) != -1;
        int action = lsoft ? FIRE : back || hangup ? 0 : getGameAction(key);
        // Keypad digits first: handsets differ in which digits they report as game actions.
        if (key == KEY_NUM2) action = UP;
        else if (key == KEY_NUM8) action = DOWN;
        else if (key == KEY_NUM4) action = LEFT;
        else if (key == KEY_NUM6) action = RIGHT;
        else if (key == KEY_NUM5) action = FIRE;
        boolean fire = action == FIRE;
        boolean upDown = action == UP || action == DOWN;

        if (hangup) {
            // The end key asks to leave from anywhere; in play the round pauses first.
            if (state == PLAY) pause();
            if (state != ASK) askFor(ASK_EXIT);
            repaint();
            return;
        }
        switch (state) {
            case LOGO:
                toTitle();
                break;
            case TITLE:
                toMenu(resume != null ? RESUME : GO);
                sound.play(Sound.MOVE);
                break;
            case MENU:
                if (upDown) {
                    menu = (menu + (action == UP ? items.length - 1 : 1)) % items.length;
                    sound.play(Sound.MOVE);
                } else if (fire) {
                    choose(items[menu]);
                } else if (back) {
                    askFor(ASK_EXIT);
                }
                break;
            case LEVEL:
                if (upDown || action == LEFT || action == RIGHT) {
                    row = 1 - row;
                    sound.play(Sound.MOVE);
                } else if (fire) {
                    level = row;
                    start();
                } else if (back) {
                    state = MENU;
                }
                break;
            case PLAY:
                if (back || lsoft) pause();
                else key(action);
                break;
            case PAUSE:
                if (upDown) {
                    menu = (menu + (action == UP ? PAUSED.length - 1 : 1)) % PAUSED.length;
                    sound.play(Sound.MOVE);
                } else if (fire) {
                    choose(PAUSED[menu]);
                } else if (back) {
                    unpause();
                }
                break;
            case ASK:
                if (fire) {
                    if (ask == ASK_EXIT) {
                        exit();
                    } else if (ask == ASK_HOME) {
                        toMenu(RESUME);
                    } else {
                        best[0] = best[1] = 0;
                        save();
                        toast("기록을 지웠어요");
                        state = askBack;
                    }
                } else if (back) {
                    state = askBack;
                }
                break;
            case OVER:
                // A short lock so the key that ended the round does not also start the next one.
                if (System.currentTimeMillis() - overAt < 800) break;
                if (fire) start();
                else if (back) toMenu(GO);
                break;
            case SETTINGS:
                if (back) {
                    save();
                    state = from;
                } else if (upDown) {
                    row = (row + (action == UP ? 3 : 1)) % 4;
                } else if (fire || action == LEFT || action == RIGHT) {
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
                if (upDown) {
                    int n = helpTopics().length;
                    topic = (topic + (action == UP ? n - 1 : 1)) % n;
                    sound.play(Sound.MOVE);
                } else if (fire) {
                    page = topic;
                    state = PAGE;
                } else if (back) {
                    state = from;
                }
                break;
            case PAGE: {
                int n = helpTopics().length;
                if (back || (fire && page == n - 1)) {
                    topic = page;
                    state = HELP;
                } else if ((action == RIGHT || fire) && page < n - 1) {
                    page++;
                } else if (action == LEFT && page > 0) {
                    page--;
                }
                break;
            }
            case SCORES:
                if (fire) askFor(ASK_RESET);
                else if (back) toMenu(BEST);
                break;
            default: // CREDITS
                if (back || fire) toMenu(ABOUT);
        }
        repaint();
    }

    protected void paint(Graphics g) {
        int w = getWidth();
        int h = getHeight();
        int bar = lineH + 10;
        g.setColor(SURFACE);
        g.fillRect(0, 0, w, h);
        String left, right;
        synchronized (this) {
            int st = state == ASK ? askBack : state;
            paintScreen(g, st, w, h - bar);
            if (state == ASK) {
                if (ask == ASK_EXIT) ask(g, w, h - bar, "나가시겠어요?", null);
                else if (ask == ASK_HOME) ask(g, w, h - bar, "처음 화면으로 갈까요?", "이 판은 이어하기로 남아요");
                else ask(g, w, h - bar, "최고 기록을 지울까요?", null);
            }
            String[] s = softLabels();
            left = s[0];
            right = s[1];
        }
        if (left != null || right != null) {
            g.setColor(WHITE);
            g.fillRect(0, h - bar, w, bar);
            g.setColor(SURFACE2);
            g.drawLine(0, h - bar, w, h - bar);
            g.setColor(TEXT);
            if (left != null) bold(g, left, 10, h - bar + 5, Graphics.TOP | Graphics.LEFT);
            if (right != null) bold(g, right, w - 11, h - bar + 5, Graphics.TOP | Graphics.RIGHT);
        }
        if (toast != null && System.currentTimeMillis() < toastUntil) {
            int tw = Font.getDefaultFont().stringWidth(toast) + 20;
            g.setColor(TEXT);
            g.fillRoundRect((w - tw) / 2, h - bar - lineH - 22, tw, lineH + 10, 12, 12);
            g.setColor(WHITE);
            g.drawString(toast, w / 2, h - bar - lineH - 17, Graphics.TOP | Graphics.HCENTER);
        }
    }

    /** What the two soft keys do on this screen: left, right (null = nothing shown). */
    private String[] softLabels() {
        switch (state) {
            case LOGO:
            case TITLE:
                return new String[] {null, null};
            case PLAY:
                return new String[] {"메뉴", null};
            case PAUSE:
                return new String[] {"선택", "계속"};
            case ASK:
                return new String[] {"예", "아니오"};
            case LEVEL:
                return new String[] {"시작", "뒤로"};
            case OVER:
                return new String[] {"다시", "메뉴"};
            case SETTINGS:
                return new String[] {"바꾸기", "뒤로"};
            case PAGE:
                return new String[] {page < helpTopics().length - 1 ? "다음" : "목차", "목차"};
            case SCORES:
                return new String[] {"지우기", "뒤로"};
            case CREDITS:
                return new String[] {null, "뒤로"};
            default: // MENU, HELP
                return new String[] {"선택", "뒤로"};
        }
    }

    /** One screen, drawn above the soft-key bar ({@code h} is its top). */
    private void paintScreen(Graphics g, int st, int w, int h) {
        if (st == LOGO) {
            paintLogo(g, w, h);
        } else if (st == TITLE) {
            paintTitle(g, w, h - lineH - 16);
            if ((frame / 3) % 2 == 0) {
                g.setColor(TEXT);
                bold(g, "아무 키나 누르세요", w / 2, h - lineH - 12, Graphics.TOP | Graphics.HCENTER);
            }
        } else if (st == MENU || st == LEVEL) {
            paintMenu(g, w, h);
            if (st == LEVEL) paintLevel(g, w, h);
        } else if (st == SETTINGS) {
            paintSettings(g, w, h);
        } else if (st == HELP) {
            paintTopics(g, w, h);
        } else if (st == PAGE) {
            paintHelpPage(g, w, h);
        } else if (st == SCORES) {
            paintScores(g, w, h);
        } else if (st == CREDITS) {
            paintCredits(g, w, h);
        } else {
            paintPlay(g, w, h, st == PAUSE);
            if (st == PAUSE) paintPause(g, w, h);
            if (st == OVER) paintOver(g, w, h);
        }
    }

    /** The maker's mark: three pebbles settle into a cairn, then the name. */
    private void paintLogo(Graphics g, int w, int h) {
        g.setColor(WHITE);
        g.fillRect(0, 0, w, h);
        int shown = (seen ? LOGO_AGAIN : LOGO_FIRST) - logoLeft; // loop turns so far
        int cx = w / 2, base = h / 2 + 4;
        int[] pw = {64, 46, 30}, ph = {26, 20, 15}, col = {0x9AA5B4, 0xB9C2CE, 0xD5DBE3};
        int y = base;
        for (int i = 0; i < 3; i++) {
            y -= ph[i] - 3;
            if (seen || shown > i) {
                g.setColor(col[i]);
                g.fillArc(cx - pw[i] / 2, y, pw[i], ph[i], 0, 360);
            }
        }
        if (seen || shown > 3) {
            g.setColor(TEXT);
            bold(g, "otterpebble", cx, base + 34, Graphics.TOP | Graphics.HCENTER);
        }
    }

    /** A title bar, for the screens under the menus. */
    private int header(Graphics g, int w, String title, String side) {
        int bar = lineH + 12;
        g.setColor(WHITE);
        g.fillRect(0, 0, w, bar);
        g.setColor(SURFACE2);
        g.drawLine(0, bar, w, bar);
        g.setColor(TEXT);
        bold(g, title, w / 2, 6, Graphics.TOP | Graphics.HCENTER);
        if (side != null) {
            g.setColor(SUB);
            g.drawString(side, w - 10, 6, Graphics.TOP | Graphics.RIGHT);
        }
        return bar;
    }

    /** A vertical list with icons; the chosen row is taller and dark. */
    private void list(Graphics g, int w, int y, String[] labels, int[] icons, int sel) {
        int rh = lineH + 8;
        for (int i = 0; i < labels.length; i++) {
            int x = 22, rw = w - 44, bh = i == sel ? rh + 8 : rh;
            if (i == sel) {
                g.setColor(TEXT);
                g.fillRoundRect(x - 4, y, rw + 8, bh, 20, 20);
            } else {
                card(g, x, y, rw, bh, 16);
            }
            int ic = x + 8 + (bh - 4) / 2, cy = y + bh / 2;
            icon(g, icons[i], ic, cy, i == sel ? bh - 8 : bh - 6);
            g.setColor(i == sel ? WHITE : TEXT);
            if (i == sel) bold(g, labels[i], ic + bh / 2 + 10, y + (bh - lineH) / 2, Graphics.TOP | Graphics.LEFT);
            else g.drawString(labels[i], ic + bh / 2 + 10, y + (bh - lineH) / 2, Graphics.TOP | Graphics.LEFT);
            if (i == sel) {
                g.setColor(WHITE);
                play(g, x + rw - 12, cy, 5);
            }
            y += bh + 4;
        }
    }

    /** A menu icon: a pastel disc of diameter {@code d} with a dark glyph on it. */
    private void icon(Graphics g, int id, int cx, int cy, int d) {
        g.setColor(ICON_BG[id]);
        g.fillArc(cx - d / 2, cy - d / 2, d, d, 0, 360);
        g.setColor(TEXT);
        int r = d / 4;
        if (id == GO) {
            play(g, cx - r / 2, cy, r);
        } else if (id == RESUME) {
            g.fillRect(cx - r, cy - r, 2, 2 * r + 1);
            play(g, cx - r + 3, cy, r);
        } else if (id == HOWTO) {
            bold(g, "?", cx, cy - lineH / 2, Graphics.TOP | Graphics.HCENTER);
        } else if (id == SETUP) {
            // a gear: a disc with four teeth and a hole
            g.fillRect(cx - 1, cy - r - 2, 3, 2 * r + 5);
            g.fillRect(cx - r - 2, cy - 1, 2 * r + 5, 3);
            g.fillArc(cx - r, cy - r, 2 * r + 1, 2 * r + 1, 0, 360);
            g.setColor(ICON_BG[id]);
            g.fillArc(cx - r / 2, cy - r / 2, r + 1, r + 1, 0, 360);
        } else if (id == BEST) {
            // a cup: bowl, stem, foot
            g.fillRoundRect(cx - r, cy - r, 2 * r + 1, r + 3, 4, 4);
            g.fillRect(cx - 1, cy, 3, r);
            g.fillRect(cx - r + 1, cy + r - 1, 2 * r - 1, 2);
        } else if (id == ABOUT) {
            g.fillRect(cx - 1, cy - r - 1, 3, 3);
            g.fillRect(cx - 1, cy - r + 3, 3, 2 * r - 2);
        } else if (id == EXIT) {
            // a door with an arrow leaving it
            g.drawRect(cx - r, cy - r, r + 1, 2 * r);
            g.fillRect(cx - r + r / 2, cy - 1, r + r / 2 + 1, 2);
            play(g, cx + r - 1, cy, 3);
        } else { // HOME: a roof over a box
            for (int i = 0; i <= r; i++) g.drawLine(cx - i, cy - r + i - 1, cx + i, cy - r + i - 1);
            g.fillRect(cx - r + 2, cy, 2 * r - 3, r);
        }
    }

    private void paintMenu(Graphics g, int w, int h) {
        int cx = w / 2;
        paintEmblem(g, cx, 34);
        g.setColor(TEXT);
        bold(g, name(), cx, 64, Graphics.TOP | Graphics.HCENTER);
        String[] labels = new String[items.length];
        for (int i = 0; i < items.length; i++) labels[i] = MAIN[items[i]];
        list(g, w, 70 + lineH, labels, items, state == MENU || state == ASK ? menu : -1);
    }

    /** The level card over the menu: 쉬움 or 보통, then 5 starts. */
    private void paintLevel(Graphics g, int w, int h) {
        int ch = 4 * lineH + 60, y = (h - ch) / 2;
        g.setColor(TEXT);
        g.fillRoundRect(18, y - 2, w - 36, ch + 4, 22, 22);
        g.setColor(WHITE);
        g.fillRoundRect(20, y, w - 40, ch, 20, 20);
        g.setColor(TEXT);
        bold(g, "난이도를 골라요", w / 2, y + 12, Graphics.TOP | Graphics.HCENTER);
        for (int i = 0; i < 2; i++) {
            int bx = w / 2 - 94 + i * 100, by = y + lineH + 24;
            if (i == row) {
                g.setColor(TEXT);
                g.fillRoundRect(bx - 2, by - 2, 92, lineH + 20, 18, 18);
                g.setColor(WHITE);
                bold(g, LEVELS[i], bx + 44, by + 7, Graphics.TOP | Graphics.HCENTER);
            } else {
                g.setColor(SURFACE2);
                g.fillRoundRect(bx, by, 88, lineH + 16, 16, 16);
                g.setColor(TEXT);
                g.drawString(LEVELS[i], bx + 44, by + 7, Graphics.TOP | Graphics.HCENTER);
            }
        }
        g.setColor(SUB);
        g.drawString(LEVEL_NOTES[row], w / 2, y + 2 * lineH + 48, Graphics.TOP | Graphics.HCENTER);
        g.drawString("4 6 고르고 5 로 시작해요", w / 2, y + 3 * lineH + 52, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintSettings(Graphics g, int w, int h) {
        int y = header(g, w, "설정", null) + 16;
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
        String note = row == 3 ? MOTION_NOTES[motion] : row == 2 && from == PAUSE ? "다음 판부터 바뀌어요" : LEVEL_NOTES[level];
        g.drawString(note, w / 2, y, Graphics.TOP | Graphics.HCENTER);
        g.drawString("2 8 로 고르고 5 로 바꿔요", w / 2, h - lineH - 8, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintTopics(Graphics g, int w, int h) {
        String[] t = helpTopics();
        int y = header(g, w, "게임 방법", null) + 8;
        int rh = lineH + 8;
        for (int i = 0; i < t.length; i++) {
            if (i == topic) {
                g.setColor(TEXT);
                g.fillRoundRect(16, y, w - 32, rh, 16, 16);
            } else {
                card(g, 18, y, w - 36, rh, 14);
            }
            g.setColor(i == topic ? WHITE : SUB);
            g.drawString((i + 1) + ".", 30, y + 4, Graphics.TOP | Graphics.LEFT);
            g.setColor(i == topic ? WHITE : TEXT);
            if (i == topic) bold(g, t[i], 54, y + 4, Graphics.TOP | Graphics.LEFT);
            else g.drawString(t[i], 54, y + 4, Graphics.TOP | Graphics.LEFT);
            y += rh + 3;
        }
    }

    private void paintHelpPage(Graphics g, int w, int h) {
        String[] t = helpTopics();
        int top = header(g, w, t[page], (page + 1) + "/" + t.length);
        int bottom = h - lineH - 14;
        paintHelp(g, page, top + 8, w, bottom - 4);
        for (int i = 0; i < t.length; i++) {
            g.setColor(i == page ? TEXT : 0xC9CED6);
            g.fillArc(w / 2 - t.length * 6 + i * 12 + 2, bottom, 7, 7, 0, 360);
        }
        g.setColor(SUB);
        g.drawString("4 6 넘기고 CLR 목차로", w / 2, h - lineH - 4, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintScores(Graphics g, int w, int h) {
        int y = header(g, w, "최고 기록", null) + 24;
        for (int i = 0; i < 2; i++) {
            card(g, 24, y, w - 48, 2 * lineH + 22, 16);
            g.setColor(SUB);
            g.drawString(LEVELS[i], w / 2, y + 8, Graphics.TOP | Graphics.HCENTER);
            g.setColor(TEXT);
            bold(g, best[i] + " 점", w / 2, y + lineH + 12, Graphics.TOP | Graphics.HCENTER);
            y += 2 * lineH + 34;
        }
        g.setColor(SUB);
        g.drawString("5 를 누르면 기록을 지울 수 있어요", w / 2, h - lineH - 8, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintCredits(Graphics g, int w, int h) {
        int y = header(g, w, "제작자 정보", null);
        paintEmblem(g, w / 2, y + 36);
        String version = midlet == null ? null : midlet.getAppProperty("MIDlet-Version");
        String[][] rows = {{"게임", name()}, {"버전", version == null ? "?" : version}, {"만든 곳", "otterpebble"}, {"라이선스", "MIT"}};
        y += 72;
        int rh = lineH + 8;
        card(g, 16, y, w - 32, rows.length * rh + 8, 16);
        for (int i = 0; i < rows.length; i++) {
            int ry = y + 6 + i * rh;
            g.setColor(SUB);
            g.drawString(rows[i][0], 30, ry + 4, Graphics.TOP | Graphics.LEFT);
            g.setColor(TEXT);
            bold(g, rows[i][1], w - 31, ry + 4, Graphics.TOP | Graphics.RIGHT);
        }
        y += rows.length * rh + 18;
        g.setColor(SUB);
        g.drawString("그림, 소리, 글꼴을 빌려 오지 않고", w / 2, y, Graphics.TOP | Graphics.HCENTER);
        g.drawString("모두 새로 만들었어요", w / 2, y + lineH + 2, Graphics.TOP | Graphics.HCENTER);
        g.setColor(ACCENT);
        g.drawString("featurephone.otterpebble.com", w / 2, y + 2 * lineH + 14, Graphics.TOP | Graphics.HCENTER);
    }

    private void paintPlay(Graphics g, int w, int h, boolean hidden) {
        int bar = lineH + 10;
        int top = bar + 10;
        if (!hidden) paintBoard(g, top, w, h);
        g.setColor(WHITE);
        g.fillRect(0, 0, w, top);
        g.setColor(TEXT);
        bold(g, "점수 " + score, 8, 5, Graphics.TOP | Graphics.LEFT);
        g.setColor(SUB);
        g.drawString("최고 " + best[playLevel], w - 8, 5, Graphics.TOP | Graphics.RIGHT);

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

    /** The pause menu: the board is covered (no studying it while the clock is stopped). */
    private void paintPause(Graphics g, int w, int h) {
        int top = 2 * lineH + 20;
        g.setColor(SURFACE2);
        g.fillRoundRect(8, top, w - 16, h - top - 6, 18, 18);
        g.setColor(TEXT);
        bold(g, "잠깐 쉬어요", w / 2, top + 12, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("시간이 멈췄어요", w / 2, top + lineH + 16, Graphics.TOP | Graphics.HCENTER);
        list(g, w, top + 2 * lineH + 30, PAUSED_LABELS, PAUSED, state == PAUSE ? menu : -1);
    }

    /** How many frames the end card's score takes to count up. */
    static final int COUNT_FRAMES = 6;

    /** The yes/no card: 5 = yes, CLR = no. */
    private void ask(Graphics g, int w, int h, String question, String note) {
        int cw = w - 40;
        int ch = (note == null ? 3 : 4) * lineH + 40;
        int x = 20, y = (h - ch) / 2;
        g.setColor(TEXT);
        g.fillRoundRect(x - 2, y - 2, cw + 4, ch + 4, 22, 22);
        g.setColor(WHITE);
        g.fillRoundRect(x, y, cw, ch, 20, 20);
        g.setColor(TEXT);
        bold(g, question, w / 2, y + 14, Graphics.TOP | Graphics.HCENTER);
        if (note != null) {
            g.setColor(SUB);
            g.drawString(note, w / 2, y + lineH + 20, Graphics.TOP | Graphics.HCENTER);
        }
        int by = y + ch - lineH - 18;
        g.setColor(TEXT);
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
        bold(g, "시간이 다 됐어요", cx, y + 12, Graphics.TOP | Graphics.HCENTER);
        // The score counts up over a few frames; a new best then blinks. Motion off shows it at once.
        int age = frame - overFrame;
        boolean counting = motion == 0 && age < COUNT_FRAMES;
        bold(g, (counting ? score * age / COUNT_FRAMES : score) + " 점", cx, y + lineH + 20, Graphics.TOP | Graphics.HCENTER);
        if (!counting && !(newBest && motion == 0 && (age / 2) % 2 == 1)) {
            g.setColor(newBest ? ACCENT : SUB);
            g.drawString(newBest ? "새 최고 기록이에요!" : "최고 " + best[playLevel], cx, y + 2 * lineH + 26, Graphics.TOP | Graphics.HCENTER);
        }
        g.setColor(SURFACE2);
        g.fillRoundRect(cx - 60, y + 3 * lineH + 34, 120, lineH + 8, 16, 16);
        g.setColor(TEXT);
        g.drawString("5 다시하기", cx, y + 3 * lineH + 38, Graphics.TOP | Graphics.HCENTER);
        g.setColor(SUB);
        g.drawString("CLR 메뉴로", cx, y + 4 * lineH + 44, Graphics.TOP | Graphics.HCENTER);
    }

    static void putInt(byte[] b, int at, int v) {
        b[at] = (byte) (v >>> 24);
        b[at + 1] = (byte) (v >>> 16);
        b[at + 2] = (byte) (v >>> 8);
        b[at + 3] = (byte) v;
    }

    static int getInt(byte[] b, int at) {
        return ((b[at] & 0xFF) << 24) | ((b[at + 1] & 0xFF) << 16) | ((b[at + 2] & 0xFF) << 8) | (b[at + 3] & 0xFF);
    }

    // ---- one record, named after the game: best easy · best normal (4 bytes each), sound · vibe · level · motion,
    // logo seen, then the paused round. Older 11- and 12-byte records still load (motion on, logo unseen, no round). ----
    private void load() {
        try {
            RecordStore rs = RecordStore.openRecordStore(name(), true);
            if (rs.getNumRecords() > 0) {
                byte[] b = rs.getRecord(1);
                if (b != null && b.length >= 11) {
                    best[0] = getInt(b, 0);
                    best[1] = getInt(b, 4);
                    sound.enabled = b[8] != 0;
                    vibrate = b[9] != 0;
                    level = b[10] == 0 ? 0 : 1;
                    if (b.length >= 12 && b[11] >= 0 && b[11] < 3) motion = b[11];
                    if (b.length >= 13) seen = b[12] != 0;
                    if (b.length > 13) {
                        resume = new byte[b.length - 13];
                        System.arraycopy(b, 13, resume, 0, resume.length);
                    }
                }
            }
            rs.closeRecordStore();
        } catch (Exception e) {
            // no storage: defaults, and nothing lasts past closing the game
        }
    }

    private void save() {
        byte[] b = new byte[13 + (resume == null ? 0 : resume.length)];
        putInt(b, 0, best[0]);
        putInt(b, 4, best[1]);
        b[8] = (byte) (sound.enabled ? 1 : 0);
        b[9] = (byte) (vibrate ? 1 : 0);
        b[10] = (byte) level;
        b[11] = (byte) motion;
        b[12] = (byte) (seen ? 1 : 0);
        if (resume != null) System.arraycopy(resume, 0, b, 13, resume.length);
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
                canvas.midlet = this;
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
