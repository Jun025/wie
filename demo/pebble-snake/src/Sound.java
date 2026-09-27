/*
 * Pebble Snake — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * The engine plays SMAF ("application/vnd.smaf"), the format Korean handsets shipped
 * with, and nothing else. Rather than ship binary sound files, each jingle is a short
 * note list assembled into a one-track SMAF (MA-3 "mobile standard, no compression")
 * at start-up. Only the chunks the format requires are written: MMMD > CNTI + MTR0 >
 * Mtsq, then a two-byte CRC field.
 */

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import javax.microedition.media.Manager;
import javax.microedition.media.Player;

final class Sound {
    // A tick is 4 ms (timebase code 0x02). Notes are {midiNote, lengthInTicks}; note 0 is a rest.
    static final int[][] START = {{72, 25}, {76, 25}, {79, 25}, {84, 60}};
    static final int[][] EAT = {{84, 12}, {91, 20}};
    static final int[][] TURN = {{67, 6}};
    static final int[][] OVER = {{79, 30}, {74, 30}, {71, 30}, {67, 90}};
    static final int[][] PAUSE = {{72, 15}, {67, 20}};

    private final byte[][] clips = new byte[5][];
    private Player current;
    boolean enabled = true;

    Sound() {
        clips[0] = smaf(START, 80);
        clips[1] = smaf(EAT, 13);
        clips[2] = smaf(TURN, 115);
        clips[3] = smaf(OVER, 80);
        clips[4] = smaf(PAUSE, 11);
    }

    void play(int clip) {
        if (!enabled) {
            return;
        }
        try {
            if (current != null) {
                current.close();
            }
            current = Manager.createPlayer(new ByteArrayInputStream(clips[clip]), "application/vnd.smaf");
            current.realize();
            current.prefetch();
            current.start();
        } catch (Exception e) {
            // A handset (or host) without sound keeps playing the game in silence.
            current = null;
        }
    }

    void stop() {
        if (current != null) {
            current.close();
            current = null;
        }
    }

    private static byte[] smaf(int[][] notes, int program) {
        ByteArrayOutputStream seq = new ByteArrayOutputStream();
        seq.write(0); // delta
        seq.write(0xC0); // program change, channel 0
        seq.write(program & 0x7F);
        int pending = 0;
        for (int i = 0; i < notes.length; i++) {
            int note = notes[i][0];
            int len = notes[i][1];
            if (note == 0) {
                pending += len;
                continue;
            }
            varNum(seq, pending);
            seq.write(0x90); // note with velocity, channel 0
            seq.write(note);
            seq.write(100);
            varNum(seq, len * 9 / 10); // gate: leave a short gap between notes
            pending = len;
        }
        varNum(seq, pending);
        seq.write(0xFF); // end of sequence
        seq.write(0x2F);
        seq.write(0x00);
        byte[] mtsq = seq.toByteArray();

        ByteArrayOutputStream track = new ByteArrayOutputStream();
        track.write(2); // format: mobile standard, no compression
        track.write(0); // sequence type: stream
        track.write(0x02); // duration timebase: 4 ms
        track.write(0x02); // gate timebase: 4 ms
        for (int ch = 0; ch < 16; ch++) {
            track.write(ch == 0 ? 0x01 : 0x00); // channel 0 = melody
        }
        chunk(track, "Mtsq", mtsq);
        byte[] mtr = track.toByteArray();

        ByteArrayOutputStream body = new ByteArrayOutputStream();
        chunk(body, "CNTI", new byte[] {0, 0, 0, 0, 0});
        // "MTR" + track number 0. Not written as the literal "MTR\0": class files store
        // strings as modified UTF-8, which encodes NUL as C0 80, and the engine's class
        // parser rejects that (ClassFormatError) — measured on the first build.
        chunk(body, "MTR ", mtr, 0); // last tag byte = track 0
        byte[] inner = body.toByteArray();

        ByteArrayOutputStream file = new ByteArrayOutputStream();
        writeTag(file, "MMMD");
        u32(file, inner.length + 2);
        file.write(inner, 0, inner.length);
        file.write(0); // CRC field — present, not checked by the player
        file.write(0);
        return file.toByteArray();
    }

    private static void chunk(ByteArrayOutputStream out, String tag, byte[] data) {
        chunk(out, tag, data, tag.charAt(3));
    }

    private static void chunk(ByteArrayOutputStream out, String tag, byte[] data, int lastTagByte) {
        for (int i = 0; i < 3; i++) {
            out.write(tag.charAt(i));
        }
        out.write(lastTagByte);
        u32(out, data.length);
        out.write(data, 0, data.length);
    }

    private static void writeTag(ByteArrayOutputStream out, String tag) {
        for (int i = 0; i < 4; i++) {
            out.write(tag.charAt(i));
        }
    }

    private static void u32(ByteArrayOutputStream out, int v) {
        out.write(v >>> 24);
        out.write(v >>> 16);
        out.write(v >>> 8);
        out.write(v);
    }

    private static void varNum(ByteArrayOutputStream out, int v) {
        if (v >= 0x80) {
            out.write(0x80 | ((v >> 7) & 0x7F));
        }
        out.write(v & 0x7F);
    }
}
