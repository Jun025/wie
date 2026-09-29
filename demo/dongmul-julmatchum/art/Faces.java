/*
 * 동물줄맞춤 — Copyright (c) 2026 otterpebble. MIT License (see ../LICENSE).
 *
 * Build-time only (never packaged): draws the six animal faces with Java2D anti-aliasing
 * and writes them as PNGs into the jar's class directory. Why here and not at run time:
 * the engine's Graphics has no anti-aliasing and the screen is a fixed 240x320, so a
 * face drawn with fillArc at 30px has stair-stepped edges; an image drawn once here with
 * soft (alpha) edges is blended by the engine's drawImage and reads smooth on the device.
 *
 * Run by ../../arcade-common/build-game.sh:  java Faces.java <class-dir>
 * Output: /f<kind>_<size>.png for kind 0..5 and size 30 (board) and 56 (home, help).
 * Every face is drawn in a 100x100 box, then scaled. Same JDK -> same bytes.
 *
 * Style (kept away from existing mascot sets on purpose): flat muted pastels from the
 * otterpebble palette, small dot eyes with no eye-whites, no blush, no mouth expression
 * beyond a short line. Kinds differ by silhouette as well as colour, so the board reads
 * without colour: round ears on top / tiny side ears on a wide head / pointed ears /
 * long upright ears / no ears with a face mask / eye bumps on top.
 */

import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.Shape;
import java.awt.geom.Area;
import java.awt.geom.Ellipse2D;
import java.awt.geom.Path2D;
import java.awt.geom.RoundRectangle2D;
import java.awt.image.BufferedImage;
import java.io.File;
import javax.imageio.ImageIO;

public class Faces {

    public static void main(String[] args) throws Exception {
        for (int size : new int[] {30, 56}) {
            for (int k = 0; k < 6; k++) {
                BufferedImage img = new BufferedImage(size, size, BufferedImage.TYPE_INT_ARGB);
                Graphics2D g = img.createGraphics();
                g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
                g.setRenderingHint(RenderingHints.KEY_STROKE_CONTROL, RenderingHints.VALUE_STROKE_PURE);
                g.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
                g.scale(size / 100.0, size / 100.0);
                face(g, k);
                g.dispose();
                ImageIO.write(img, "png", new File(args[0], "f" + k + "_" + size + ".png"));
            }
        }
    }

    static Shape oval(double cx, double cy, double w, double h) {
        return new Ellipse2D.Double(cx - w / 2, cy - h / 2, w, h);
    }

    /** Fills the silhouette with a darker rim first, so the face has a clean edge on white. */
    static void body(Graphics2D g, Area a, int fill, int rim) {
        g.setColor(new Color(rim));
        g.setStroke(new BasicStroke(6f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        g.draw(a);
        g.setColor(new Color(fill));
        g.fill(a);
    }

    static void fill(Graphics2D g, Shape s, int rgb) {
        g.setColor(new Color(rgb));
        g.fill(s);
    }

    static void eyes(Graphics2D g, double y, double gap, double d) {
        fill(g, oval(50 - gap, y, d, d * 1.15), 0x2B2F38);
        fill(g, oval(50 + gap, y, d, d * 1.15), 0x2B2F38);
    }

    static void line(Graphics2D g, int rgb, float w, double x1, double y1, double x2, double y2) {
        g.setColor(new Color(rgb));
        g.setStroke(new BasicStroke(w, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
        g.draw(new java.awt.geom.Line2D.Double(x1, y1, x2, y2));
    }

    static void face(Graphics2D g, int k) {
        switch (k) {
            case 0: { // 곰 bear — honey, round ears on top, pale muzzle
                Area a = new Area(oval(50, 56, 78, 72));
                a.add(new Area(oval(24, 26, 28, 28)));
                a.add(new Area(oval(76, 26, 28, 28)));
                body(g, a, 0xE2A857, 0xAE7A30);
                fill(g, oval(24, 27, 14, 14), 0xF1CE92);
                fill(g, oval(76, 27, 14, 14), 0xF1CE92);
                fill(g, oval(50, 70, 36, 26), 0xF6E3C0);
                fill(g, oval(50, 63, 14, 9), 0x4A3524);
                line(g, 0x4A3524, 2.6f, 50, 66, 50, 74);
                eyes(g, 50, 16, 8);
                break;
            }
            case 1: { // 수달 otter — chocolate, wide head, tiny side ears, cream cheeks + whiskers
                Area a = new Area(oval(50, 56, 88, 70));
                a.add(new Area(oval(12, 44, 16, 16)));
                a.add(new Area(oval(88, 44, 16, 16)));
                body(g, a, 0x94664C, 0x5F3F2D);
                Area m = new Area(oval(39, 70, 30, 26));
                m.add(new Area(oval(61, 70, 30, 26)));
                g.setColor(new Color(0xEEDCC8));
                g.fill(m);
                fill(g, new RoundRectangle2D.Double(41, 58, 18, 11, 9, 9), 0x3A2A20);
                for (int s = -1; s <= 1; s += 2) {
                    for (int i = 0; i < 3; i++) fill(g, oval(50 + s * (19 + i * 5), 72 + (i % 2) * 4, 3.2, 3.2), 0x8C6F58);
                }
                eyes(g, 46, 17, 8);
                break;
            }
            case 2: { // 고양이 cat — blue-grey, pointed ears, forehead stripes
                Path2D ears = new Path2D.Double();
                ears.moveTo(14, 44);
                ears.lineTo(20, 8);
                ears.lineTo(44, 28);
                ears.closePath();
                ears.moveTo(86, 44);
                ears.lineTo(80, 8);
                ears.lineTo(56, 28);
                ears.closePath();
                Area a = new Area(oval(50, 58, 82, 68));
                a.add(new Area(ears));
                body(g, a, 0x8B9BB9, 0x5A6A88);
                Path2D inner = new Path2D.Double();
                inner.moveTo(22, 36);
                inner.lineTo(25, 18);
                inner.lineTo(36, 29);
                inner.closePath();
                inner.moveTo(78, 36);
                inner.lineTo(75, 18);
                inner.lineTo(64, 29);
                inner.closePath();
                g.setColor(new Color(0xC9D2E3));
                g.fill(inner);
                line(g, 0x66779A, 4f, 50, 30, 50, 40);
                line(g, 0x66779A, 4f, 41, 32, 43, 40);
                line(g, 0x66779A, 4f, 59, 32, 57, 40);
                Path2D nose = new Path2D.Double();
                nose.moveTo(44, 63);
                nose.lineTo(56, 63);
                nose.lineTo(50, 70);
                nose.closePath();
                g.setColor(new Color(0xD98C86));
                g.fill(nose);
                for (int s = -1; s <= 1; s += 2) {
                    line(g, 0x5A6A88, 2.2f, 50 + s * 14, 68, 50 + s * 38, 64);
                    line(g, 0x5A6A88, 2.2f, 50 + s * 14, 72, 50 + s * 38, 76);
                }
                eyes(g, 52, 17, 8);
                break;
            }
            case 3: { // 토끼 rabbit — pale lilac, long upright ears
                Area a = new Area(oval(50, 64, 74, 62));
                a.add(new Area(new RoundRectangle2D.Double(26, 2, 18, 50, 18, 18)));
                a.add(new Area(new RoundRectangle2D.Double(56, 2, 18, 50, 18, 18)));
                body(g, a, 0xEDE8F7, 0x958AB8);
                fill(g, new RoundRectangle2D.Double(31, 9, 8, 34, 8, 8), 0xCDC2E8);
                fill(g, new RoundRectangle2D.Double(61, 9, 8, 34, 8, 8), 0xCDC2E8);
                fill(g, oval(50, 71, 10, 7), 0xB88FB4);
                line(g, 0x8C7FA8, 2.4f, 50, 74, 50, 80);
                line(g, 0x8C7FA8, 2.4f, 50, 80, 44, 83);
                line(g, 0x8C7FA8, 2.4f, 50, 80, 56, 83);
                eyes(g, 61, 15, 7.5);
                break;
            }
            case 4: { // 펭귄 penguin — navy dome, white face mask, orange beak, no ears
                Area a = new Area(oval(50, 54, 82, 84));
                body(g, a, 0x3E4A66, 0x252D40);
                Area m = new Area(oval(37, 58, 34, 44));
                m.add(new Area(oval(63, 58, 34, 44)));
                m.intersect(new Area(oval(50, 58, 74, 64)));
                g.setColor(Color.WHITE);
                g.fill(m);
                Path2D beak = new Path2D.Double();
                beak.moveTo(41, 64);
                beak.lineTo(59, 64);
                beak.lineTo(50, 75);
                beak.closePath();
                g.setColor(new Color(0xEE9F48));
                g.fill(beak);
                eyes(g, 53, 13, 7.5);
                break;
            }
            default: { // 개구리 frog — green, wide head, eye bumps on top
                Area a = new Area(oval(50, 62, 88, 62));
                a.add(new Area(oval(28, 32, 32, 30)));
                a.add(new Area(oval(72, 32, 32, 30)));
                body(g, a, 0x8BC98B, 0x55935B);
                fill(g, oval(28, 32, 18, 18), 0xFFFFFF);
                fill(g, oval(72, 32, 18, 18), 0xFFFFFF);
                fill(g, oval(29, 33, 9, 10), 0x2B2F38);
                fill(g, oval(71, 33, 9, 10), 0x2B2F38);
                fill(g, oval(44, 58, 3.5, 3.5), 0x3F7446);
                fill(g, oval(56, 58, 3.5, 3.5), 0x3F7446);
                g.setColor(new Color(0x3F7446));
                g.setStroke(new BasicStroke(3f, BasicStroke.CAP_ROUND, BasicStroke.JOIN_ROUND));
                g.draw(new java.awt.geom.Arc2D.Double(30, 52, 40, 24, 200, 140, java.awt.geom.Arc2D.OPEN));
            }
        }
    }
}
