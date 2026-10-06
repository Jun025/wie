//! MascotCapsule micro3D v3 data (MBAC model · MTRA action · 8-bit BMP texture) and a software
//! rasterizer for the SKT `m/XO_World` API.
//!
//! The file formats follow the reverse-engineered loader of JL-Mod (`ru.woesss.j2me.micro3d.Loader`,
//! `Action`, native `Utils.transform` — Copyright 2020-2025 Yury Kharchenko, Apache-2.0); this is an
//! independent Rust reimplementation of the subset the SKT titles use: MBAC v2/v3 (vertex format 1,
//! polygon format 1, no normals) and MTRA v2–v5 bone actions. Fixed-point units are micro3D's:
//! 4096 = 1.0 and a full turn.

use alloc::{vec, vec::Vec};

const ONE: f32 = 4096.0;

// angle in 4096-per-turn units → 4096-scaled sine, like micro3D `Util3D.sin`
pub fn isin(angle: i32) -> i32 {
    let a = angle.rem_euclid(4096);
    round(sin_turn(a as f32 / 4096.0) * ONE)
}

pub fn icos(angle: i32) -> i32 {
    isin(angle.wrapping_add(1024))
}

fn round(v: f32) -> i32 {
    if v >= 0.0 { (v + 0.5) as i32 } else { -((-v + 0.5) as i32) }
}

// sin(2π·t) for any t; reduced to a quarter wave and evaluated by a 9th order Taylor polynomial
// (error < 4e-6, well below one 4096th)
fn sin_turn(t: f32) -> f32 {
    let mut t = t - (t as i32) as f32;
    if t < 0.0 {
        t += 1.0;
    }
    let (t, sign) = if t >= 0.5 { (t - 0.5, -1.0) } else { (t, 1.0) };
    let t = if t > 0.25 { 0.5 - t } else { t };
    let x = t * core::f32::consts::TAU;
    let x2 = x * x;
    sign * x * (1.0 - x2 / 6.0 * (1.0 - x2 / 20.0 * (1.0 - x2 / 42.0 * (1.0 - x2 / 72.0))))
}

fn sqrt(v: f32) -> f32 {
    if v <= 0.0 {
        return 0.0;
    }
    let mut x = f32::from_bits((v.to_bits() >> 1) + 0x1fc0_0000);
    for _ in 0..4 {
        x = 0.5 * (x + v / x);
    }
    x
}

fn isqrt(v: i64) -> i64 {
    if v <= 0 {
        return 0;
    }
    let mut x = sqrt(v as f32) as i64;
    while x * x > v {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= v {
        x += 1;
    }
    // round to nearest, as micro3D's uSqrt does
    if v - x * x > x { x + 1 } else { x }
}

/// micro3D `AffineTrans`: 3×4 row-major, rotation 4096-scaled, translation in model units.
pub type Affine = [i32; 12];

pub const IDENTITY: Affine = [4096, 0, 0, 0, 0, 4096, 0, 0, 0, 0, 4096, 0];

pub fn mul(a: &Affine, b: &Affine) -> Affine {
    let mut r = [0i32; 12];
    for row in 0..3 {
        for col in 0..4 {
            let mut v = (0..3).map(|k| a[row * 4 + k] as i64 * b[k * 4 + col] as i64).sum::<i64>();
            v = (v + 2048) >> 12;
            if col == 3 {
                v += a[row * 4 + 3] as i64;
            }
            r[row * 4 + col] = v as i32;
        }
    }
    r
}

pub fn transform(m: &Affine, v: [i32; 3]) -> [i32; 3] {
    let mut r = [0; 3];
    for (row, out) in r.iter_mut().enumerate() {
        let d = (0..3).map(|k| m[row * 4 + k] as i64 * v[k] as i64).sum::<i64>();
        *out = (((d + 2048) >> 12) as i32).wrapping_add(m[row * 4 + 3]);
    }
    r
}

/// Rotation part only (translation kept), like `AffineTrans.rotationY/Z`.
pub fn rotation_y(m: &mut Affine, angle: i32) {
    let (s, c) = (isin(angle), icos(angle));
    [m[0], m[1], m[2], m[4], m[5], m[6], m[8], m[9], m[10]] = [c, 0, s, 0, 4096, 0, -s, 0, c];
}

pub fn rotation_z(m: &mut Affine, angle: i32) {
    let (s, c) = (isin(angle), icos(angle));
    [m[0], m[1], m[2], m[4], m[5], m[6], m[8], m[9], m[10]] = [c, -s, 0, s, c, 0, 0, 0, 4096];
}

fn cross(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [
        a[1].wrapping_mul(b[2]).wrapping_sub(a[2].wrapping_mul(b[1])),
        a[2].wrapping_mul(b[0]).wrapping_sub(a[0].wrapping_mul(b[2])),
        a[0].wrapping_mul(b[1]).wrapping_sub(a[1].wrapping_mul(b[0])),
    ]
}

// `Vector3D.unit()`
fn unit(v: [i32; 3]) -> [i32; 3] {
    let bits = v[0].unsigned_abs() | v[1].unsigned_abs() | v[2].unsigned_abs();
    let shift = bits.leading_zeros() as i32 - 17;
    let v = v.map(|c| if shift >= 0 { c.wrapping_shl(shift as u32) } else { c >> -shift });
    let len = isqrt(v.iter().map(|&c| c as i64 * c as i64).sum());
    if len == 0 {
        return [0; 3];
    }
    v.map(|c| ((c as i64 * 4096) / len) as i32)
}

/// `AffineTrans.lookAt(pos, look, up)` — `look` is a direction, not a target point.
pub fn look_at(pos: [i32; 3], look: [i32; 3], up: [i32; 3]) -> Affine {
    let mp = pos.map(|c| c.wrapping_neg() as i64);
    let x = unit(cross(look, up));
    let y = unit(cross(look, x));
    let z = unit(look);
    let mut m = [0; 12];
    for (row, axis) in [x, y, z].into_iter().enumerate() {
        m[row * 4..row * 4 + 3].copy_from_slice(&axis);
        m[row * 4 + 3] = ((mp[0] * axis[0] as i64 + mp[1] * axis[1] as i64 + mp[2] * axis[2] as i64 + 2048) >> 12) as i32;
    }
    m
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> Option<u8> {
        let v = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(v)
    }

    fn u16(&mut self) -> Option<u16> {
        let v = self.data.get(self.pos..self.pos + 2)?;
        self.pos += 2;
        Some(u16::from_le_bytes([v[0], v[1]]))
    }

    fn i16(&mut self) -> Option<i16> {
        self.u16().map(|v| v as i16)
    }

    fn u32(&mut self) -> Option<u32> {
        let v = self.data.get(self.pos..self.pos + 4)?;
        self.pos += 4;
        Some(u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
    }

    fn matrix(&mut self) -> Option<[f32; 12]> {
        let mut m = [0.0; 12];
        for (i, v) in m.iter_mut().enumerate() {
            let raw = self.i16()? as f32;
            *v = if i % 4 == 3 { raw } else { raw / ONE };
        }
        Some(m)
    }
}

pub struct Triangle {
    pub vertices: [u16; 3],
    pub uv: [[u8; 2]; 3],
    pub transparent: bool,
    pub double_face: bool,
}

struct Bone {
    vertices: usize,
    parent: i16,
    matrix: [f32; 12],
}

pub struct Model {
    vertices: Vec<[f32; 3]>,
    pub triangles: Vec<Triangle>,
    bones: Vec<Bone>,
}

/// MBAC v2/v3. Later versions carry vertex/normal/polygon format bytes this loader does not decode;
/// they return `None` so the caller can say so instead of drawing garbage.
pub fn parse_mbac(data: &[u8]) -> Option<Model> {
    let mut r = Reader { data, pos: 0 };
    if r.u8()? != b'M' || r.u8()? != b'B' {
        return None;
    }
    let version = r.u8()?;
    if r.u8()? != 0 || !(2..=3).contains(&version) {
        return None;
    }
    let num_vertices = r.u16()? as usize;
    let num_t3 = r.u16()? as usize;
    let num_t4 = r.u16()? as usize;
    let num_bones = r.u16()? as usize;

    let mut vertices = Vec::with_capacity(num_vertices);
    for _ in 0..num_vertices {
        vertices.push([r.i16()? as f32, r.i16()? as f32, r.i16()? as f32]);
    }

    let mut triangles = Vec::with_capacity(num_t3 + num_t4 * 2);
    for quad in [false, true] {
        for _ in 0..if quad { num_t4 } else { num_t3 } {
            let material = r.u16()?;
            let n = if quad { 4 } else { 3 };
            let mut idx = [0u16; 4];
            for i in idx.iter_mut().take(n) {
                *i = r.u16()?;
                if *i as usize >= num_vertices {
                    return None;
                }
            }
            let mut uv = [[0u8; 2]; 4];
            for c in uv.iter_mut().take(n) {
                *c = [r.u8()?, r.u8()?];
            }
            let transparent = material & 2 != 0;
            let double_face = material & 4 != 0;
            let tri = |a: usize, b: usize, c: usize| Triangle {
                vertices: [idx[a], idx[b], idx[c]],
                uv: [uv[a], uv[b], uv[c]],
                transparent,
                double_face,
            };
            triangles.push(tri(0, 1, 2));
            if quad {
                triangles.push(tri(2, 1, 3));
            }
        }
    }

    let mut bones = Vec::with_capacity(num_bones);
    let mut total = 0;
    for _ in 0..num_bones {
        let count = r.u16()? as usize;
        let parent = r.i16()?;
        if parent < -1 || parent as isize >= bones.len() as isize {
            return None;
        }
        bones.push(Bone {
            vertices: count,
            parent,
            matrix: r.matrix()?,
        });
        total += count;
    }
    if total != num_vertices {
        return None;
    }

    Some(Model { vertices, triangles, bones })
}

struct Track {
    keys: Vec<u16>,
    values: Vec<[f32; 3]>,
}

impl Track {
    fn read(r: &mut Reader, scale: f32, components: usize) -> Option<Self> {
        let count = r.u16()? as usize;
        let mut keys = Vec::with_capacity(count);
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            keys.push(r.u16()?);
            let mut v = [0.0; 3];
            for c in v.iter_mut().take(components) {
                *c = r.i16()? as f32 * scale;
            }
            values.push(v);
        }
        Some(Self { keys, values })
    }

    fn constant(r: &mut Reader, scale: f32, components: usize) -> Option<Self> {
        let mut v = [0.0; 3];
        for c in v.iter_mut().take(components) {
            *c = r.i16()? as f32 * scale;
        }
        Some(Self {
            keys: vec![0],
            values: vec![v],
        })
    }

    // linear interpolation between the bracketing keys; clamps past the last one
    fn get(&self, frame: f32) -> Option<[f32; 3]> {
        let last = self.keys.len().checked_sub(1)?;
        if frame >= self.keys[last] as f32 {
            return Some(self.values[last]);
        }
        let i = self.keys.iter().rposition(|&k| k as f32 <= frame)?;
        let (k0, k1) = (self.keys[i] as f32, self.keys[i + 1] as f32);
        let t = (frame - k0) / (k1 - k0);
        let (a, b) = (self.values[i], self.values[i + 1]);
        Some([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t])
    }
}

enum BoneAction {
    Matrix([f32; 12]),
    Animated {
        translate: Option<Track>,
        scale: Option<Track>,
        rotate: Track,
        roll: Option<Track>,
    },
}

struct Action {
    keyframes: u16,
    bones: Vec<BoneAction>,
}

pub struct Actions {
    actions: Vec<Action>,
}

impl Actions {
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn keyframes(&self, action: usize) -> Option<u16> {
        self.actions.get(action).map(|a| a.keyframes)
    }
}

pub fn parse_mtra(data: &[u8]) -> Option<Actions> {
    let mut r = Reader { data, pos: 0 };
    if r.u8()? != b'M' || r.u8()? != b'T' {
        return None;
    }
    let version = r.u8()?;
    if r.u8()? != 0 || !(2..=5).contains(&version) {
        return None;
    }
    let num_actions = r.u16()? as usize;
    let num_bones = r.u16()? as usize;
    r.pos += 8 * 2 + 4; // bone counts per transform type · data size — allocation hints only

    let roll = 1.0;
    let mut actions = Vec::with_capacity(num_actions);
    for _ in 0..num_actions {
        let keyframes = r.u16()?;
        let mut bones = Vec::with_capacity(num_bones);
        for _ in 0..num_bones {
            let kind = r.u8()?;
            bones.push(match kind {
                0 => BoneAction::Matrix(r.matrix()?),
                1 => BoneAction::Matrix([1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]),
                2 => {
                    let translate = Track::read(&mut r, 1.0, 3)?;
                    let scale = Track::read(&mut r, 1.0 / ONE, 3)?;
                    let rotate = Track::read(&mut r, 1.0, 3)?;
                    let roll = Track::read(&mut r, roll, 1)?;
                    BoneAction::Animated {
                        translate: Some(translate),
                        scale: Some(scale),
                        rotate,
                        roll: Some(roll),
                    }
                }
                3 => {
                    let translate = Track::constant(&mut r, 1.0, 3)?;
                    let rotate = Track::read(&mut r, 1.0, 3)?;
                    let roll = Track::constant(&mut r, roll, 1)?;
                    BoneAction::Animated {
                        translate: Some(translate),
                        scale: None,
                        rotate,
                        roll: Some(roll),
                    }
                }
                4 => {
                    let rotate = Track::read(&mut r, 1.0, 3)?;
                    let roll = Track::read(&mut r, roll, 1)?;
                    BoneAction::Animated {
                        translate: None,
                        scale: None,
                        rotate,
                        roll: Some(roll),
                    }
                }
                5 => BoneAction::Animated {
                    translate: None,
                    scale: None,
                    rotate: Track::read(&mut r, 1.0, 3)?,
                    roll: None,
                },
                6 => {
                    let translate = Track::read(&mut r, 1.0, 3)?;
                    let rotate = Track::read(&mut r, 1.0, 3)?;
                    let roll = Track::read(&mut r, roll, 1)?;
                    BoneAction::Animated {
                        translate: Some(translate),
                        scale: None,
                        rotate,
                        roll: Some(roll),
                    }
                }
                _ => return None,
            });
        }
        if version >= 5 {
            // dynamic polygon patterns: (frame u16, pattern u32) — v2/v3 models carry no patterns
            let count = r.u16()? as usize;
            for _ in 0..count {
                r.u16()?;
                r.u32()?;
            }
        }
        actions.push(Action { keyframes, bones });
    }

    Some(Actions { actions })
}

// `Action.Bone.setFrame`: rotation to the new z axis (x, y, z), then roll around it, then scale
fn animated_matrix(translate: Option<[f32; 3]>, scale: Option<[f32; 3]>, rotate: [f32; 3], roll: f32) -> [f32; 12] {
    let mut m = [0.0f32; 12];
    if let Some(t) = translate {
        [m[3], m[7], m[11]] = t;
    }
    let len = sqrt(rotate[0] * rotate[0] + rotate[1] * rotate[1] + rotate[2] * rotate[2]);
    let [x, y, z] = if len > 0.0 { rotate.map(|c| c / len) } else { [0.0, 0.0, 1.0] };
    let (xx, yy) = (x * x, y * y);
    if xx > 0.0 || yy > 0.0 {
        let a = (1.0 - z) / (yy + xx);
        let b = a * -(x * y);
        [m[0], m[1], m[2], m[4], m[5], m[6], m[8], m[9]] = [z + yy * a, b, x, b, z + xx * a, y, -x, -y];
    } else {
        [m[0], m[1], m[2], m[4], m[5], m[6], m[8], m[9]] = [1.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0];
    }
    m[10] = z;

    // roll is 4096 per turn
    let (s, c) = (sin_turn(roll / ONE), sin_turn(roll / ONE + 0.25));
    for row in 0..3 {
        let (m0, m1) = (m[row * 4], m[row * 4 + 1]);
        m[row * 4] = m0 * c + m1 * s;
        m[row * 4 + 1] = m1 * c - m0 * s;
    }

    if let Some(s) = scale {
        for row in 0..3 {
            for col in 0..3 {
                m[row * 4 + col] *= s[col];
            }
        }
    }
    m
}

fn mul_f(a: &[f32; 12], b: &[f32; 12]) -> [f32; 12] {
    let mut r = [0.0; 12];
    for row in 0..3 {
        for col in 0..4 {
            r[row * 4 + col] = (0..3).map(|k| a[row * 4 + k] * b[k * 4 + col]).sum::<f32>() + if col == 3 { a[row * 4 + 3] } else { 0.0 };
        }
    }
    r
}

/// Model-space vertices after skinning, for `action` at `frame` (16.16 fixed). `None` = rest pose.
pub fn pose(model: &Model, actions: Option<(&Actions, usize, i32)>) -> Vec<[f32; 3]> {
    let action = actions.and_then(|(a, i, f)| a.actions.get(i).map(|a| (a, f as f32 / 65536.0)));
    let mut world: Vec<[f32; 12]> = Vec::with_capacity(model.bones.len());
    let mut out = Vec::with_capacity(model.vertices.len());
    let mut vertex = 0;
    for (i, bone) in model.bones.iter().enumerate() {
        let mut m = if bone.parent < 0 {
            bone.matrix
        } else {
            mul_f(&world[bone.parent as usize], &bone.matrix)
        };
        if let Some(local) = action
            .and_then(|(a, frame)| a.bones.get(i).map(|b| (b, frame)))
            .map(|(b, frame)| match b {
                BoneAction::Matrix(m) => *m,
                BoneAction::Animated {
                    translate,
                    scale,
                    rotate,
                    roll,
                } => animated_matrix(
                    translate.as_ref().and_then(|t| t.get(frame)),
                    scale.as_ref().and_then(|t| t.get(frame)),
                    rotate.get(frame).unwrap_or([0.0, 0.0, 1.0]),
                    roll.as_ref().and_then(|t| t.get(frame)).map_or(0.0, |v| v[0]),
                ),
            })
        {
            m = mul_f(&m, &local);
        }
        for v in model.vertices.iter().skip(vertex).take(bone.vertices) {
            out.push([
                v[0] * m[0] + v[1] * m[1] + v[2] * m[2] + m[3],
                v[0] * m[4] + v[1] * m[5] + v[2] * m[6] + m[7],
                v[0] * m[8] + v[1] * m[9] + v[2] * m[10] + m[11],
            ]);
        }
        vertex += bone.vertices;
        world.push(m);
    }
    out
}

/// 8-bit palettized BMP as ARGB; palette index 0 keeps alpha 0 (micro3D's color key).
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

pub fn parse_bmp(data: &[u8]) -> Option<Texture> {
    let mut r = Reader { data, pos: 0 };
    if r.u8()? != b'B' || r.u8()? != b'M' {
        return None;
    }
    r.pos = 10;
    let mut raster = r.u32()? as usize;
    let header = r.u32()? as usize;
    let (width, height, colors, color_size) = match header {
        12 => {
            let (w, h) = (r.u16()? as i32, r.u16()? as i16 as i32);
            r.u16()?;
            if r.u16()? != 8 {
                return None;
            }
            (w, h, 256, 3)
        }
        40 => {
            let (w, h) = (r.u32()? as i32, r.u32()? as i32);
            r.u16()?;
            if r.u16()? != 8 || r.u32()? != 0 {
                return None;
            }
            r.pos += 12;
            let n = r.u32()? as usize;
            (w, h, if n == 0 || n > 256 { 256 } else { n }, 4)
        }
        _ => return None,
    };
    if width <= 0 || height == 0 {
        return None;
    }
    let palette = 14 + header;
    raster = raster.max(palette + colors * color_size);
    let (w, h) = (width as usize, height.unsigned_abs() as usize);
    let stride = w.div_ceil(4) * 4;
    let mut pixels = vec![0u32; w * h];
    for y in 0..h {
        let src_row = if height > 0 { h - 1 - y } else { y };
        for x in 0..w {
            let index = *data.get(raster + src_row * stride + x)? as usize;
            let c = data.get(palette + index * color_size..palette + index * color_size + 3)?;
            let alpha = if index == 0 { 0 } else { 0xff00_0000 };
            pixels[y * w + x] = alpha | (c[2] as u32) << 16 | (c[1] as u32) << 8 | c[0] as u32;
        }
    }
    Some(Texture { width: w, height: h, pixels })
}

/// Parallel projection `screen = center + (view · v) · scale / 4096` (micro3D `setParallelScale`),
/// z-buffered, into an ARGB buffer covering `clip` (`x, y, w, h`). Untextured models draw grey.
pub struct Frame {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    model: &Model,
    vertices: &[[f32; 3]],
    texture: Option<&Texture>,
    view: &Affine,
    scale: (i32, i32),
    center: (i32, i32),
    clip: (i32, i32, i32, i32),
) -> Option<Frame> {
    let v = |r: usize| {
        [
            view[r * 4] as f32 / ONE,
            view[r * 4 + 1] as f32 / ONE,
            view[r * 4 + 2] as f32 / ONE,
            view[r * 4 + 3] as f32,
        ]
    };
    let rows = [v(0), v(1), v(2)];
    let (sx, sy) = (scale.0 as f32 / ONE, scale.1 as f32 / ONE);
    let screen: Vec<[f32; 3]> = vertices
        .iter()
        .map(|p| {
            let [x, y, z] = rows.map(|r| r[0] * p[0] + r[1] * p[1] + r[2] * p[2] + r[3]);
            [center.0 as f32 + x * sx, center.1 as f32 + y * sy, z]
        })
        .collect();

    let (cx, cy, cw, ch) = clip;
    if cw <= 0 || ch <= 0 {
        return None;
    }
    let (width, height) = (cw as usize, ch as usize);
    let mut pixels = vec![0u32; width * height];
    let mut depth = vec![f32::INFINITY; width * height];

    for tri in &model.triangles {
        let p = tri.vertices.map(|i| screen[i as usize]);
        // CULL: front faces are clockwise on screen (y down); double-faced polygons draw both sides
        let area = (p[1][0] - p[0][0]) * (p[2][1] - p[0][1]) - (p[2][0] - p[0][0]) * (p[1][1] - p[0][1]);
        if area == 0.0 || (area < 0.0 && !tri.double_face) {
            continue;
        }
        let min_x = p.iter().fold(f32::INFINITY, |a, q| a.min(q[0])).max(cx as f32) as i32;
        let max_x = p.iter().fold(f32::NEG_INFINITY, |a, q| a.max(q[0])).min((cx + cw - 1) as f32) as i32;
        let min_y = p.iter().fold(f32::INFINITY, |a, q| a.min(q[1])).max(cy as f32) as i32;
        let max_y = p.iter().fold(f32::NEG_INFINITY, |a, q| a.max(q[1])).min((cy + ch - 1) as f32) as i32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let w0 = ((p[1][0] - px) * (p[2][1] - py) - (p[2][0] - px) * (p[1][1] - py)) / area;
                let w1 = ((p[2][0] - px) * (p[0][1] - py) - (p[0][0] - px) * (p[2][1] - py)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }
                let i = (y - cy) as usize * width + (x - cx) as usize;
                let z = w0 * p[0][2] + w1 * p[1][2] + w2 * p[2][2];
                if z >= depth[i] {
                    continue;
                }
                let color = match texture {
                    Some(t) => {
                        let u = w0 * tri.uv[0][0] as f32 + w1 * tri.uv[1][0] as f32 + w2 * tri.uv[2][0] as f32;
                        let v = w0 * tri.uv[0][1] as f32 + w1 * tri.uv[1][1] as f32 + w2 * tri.uv[2][1] as f32;
                        let (tx, ty) = ((u as usize).min(t.width - 1), (v as usize).min(t.height - 1));
                        t.pixels[ty * t.width + tx]
                    }
                    None => 0xff80_8080,
                };
                if color >> 24 == 0 && tri.transparent {
                    continue;
                }
                pixels[i] = color | 0xff00_0000;
                depth[i] = z;
            }
        }
    }

    Some(Frame {
        x: cx,
        y: cy,
        width,
        height,
        pixels,
    })
}

/// MBAC v3 of one double-sided-off quad (−10..10 in x/y, z 0) on one identity bone, uv 0..2.
#[cfg(test)]
pub(crate) fn quad_model(transparent: bool) -> Vec<u8> {
    // MBAC v3: 4 vertices, 0 triangles, 1 quad, 1 bone (identity)
    let mut d = vec![b'M', b'B', 3, 0];
    for v in [4u16, 0, 1, 1] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    for v in [-10i16, -10, 0, 10, -10, 0, -10, 10, 0, 10, 10, 0] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    let material: u16 = 1 | if transparent { 2 } else { 0 };
    for v in [material, 0, 1, 2, 3] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d.extend_from_slice(&[0, 0, 2, 0, 0, 2, 2, 2]);
    for v in [4u16, 0xffff] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    for v in [4096i16, 0, 0, 0, 0, 4096, 0, 0, 0, 0, 4096, 0] {
        d.extend_from_slice(&v.to_le_bytes());
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trig_matches_micro3d_table_points() {
        assert_eq!([isin(0), isin(1024), isin(2048), isin(3072), isin(4096)], [0, 4096, 0, -4096, 0]);
        assert_eq!(isin(512), 2896); // round(sin(45°)·4096)
        assert_eq!(icos(-1024), 0);
        assert_eq!(isin(-170), -round(sin_turn(170.0 / 4096.0) * ONE));
    }

    #[test]
    fn look_at_down_z_flips_x_and_y() {
        // camera at origin looking +z with +y up: micro3D's right = look × up = -x, down = -y
        assert_eq!(
            look_at([0, 0, 0], [0, 0, 200], [0, 4096, 0]),
            [-4096, 0, 0, 0, 0, -4096, 0, 0, 0, 0, 4096, 0]
        );
        let m = look_at([0, 0, -200], [0, 0, 200], [0, 4096, 0]);
        assert_eq!(transform(&m, [0, 0, 0]), [0, 0, 200]);
    }

    #[test]
    fn rotation_and_mul_compose() {
        let mut a = IDENTITY;
        rotation_y(&mut a, 1024);
        let mut b = IDENTITY;
        rotation_y(&mut b, -1024);
        assert_eq!(mul(&a, &b), IDENTITY);
        assert_eq!(transform(&a, [100, 0, 0]), [0, 0, -100]);
    }

    #[test]
    fn quad_renders_front_face_with_texture_and_color_key() {
        let model = parse_mbac(&quad_model(true)).unwrap();
        assert_eq!(model.triangles.len(), 2);
        let tex = Texture {
            width: 2,
            height: 2,
            pixels: vec![0x0000_0000, 0xffff_0000, 0xff00_ff00, 0xff00_00ff],
        };
        let verts = pose(&model, None);
        // identity view: +x right, +y down on screen; the quad is wound so it faces the viewer
        let frame = render(&model, &verts, Some(&tex), &IDENTITY, (4096, 4096), (20, 20), (0, 0, 40, 40)).unwrap();
        let at = |x: usize, y: usize| frame.pixels[y * 40 + x];
        assert_eq!(at(27, 12), 0xffff_0000); // texel (1, 0)
        assert_eq!(at(12, 27), 0xff00_ff00); // texel (0, 1)
        assert_eq!(at(27, 27), 0xff00_00ff); // texel (1, 1) — the quad's second triangle
        assert_eq!(at(12, 12), 0); // index 0 on a transparent polygon is the color key
        assert_eq!(at(2, 2), 0); // outside
        // seen from behind (mirrored by rotY 180°) the single-faced quad is culled
        let mut back = IDENTITY;
        rotation_y(&mut back, 2048);
        let frame = render(&model, &verts, Some(&tex), &back, (4096, 4096), (20, 20), (0, 0, 40, 40)).unwrap();
        assert!(frame.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn mtra_rotation_track_turns_the_bone() {
        // one action, one bone, type 5 (rotate only) with the new z axis = +x at every frame
        let mut d = vec![b'M', b'T', 4, 0];
        for v in [1u16, 1] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&[0; 20]);
        d.extend_from_slice(&10u16.to_le_bytes());
        d.push(5);
        for v in [1u16, 0, 4096, 0, 0] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        let actions = parse_mtra(&d).unwrap();
        assert_eq!(actions.keyframes(0), Some(10));
        let model = parse_mbac(&quad_model(false)).unwrap();
        let rest = pose(&model, None);
        let posed = pose(&model, Some((&actions, 0, 0)));
        assert_eq!(rest[1], [10.0, -10.0, 0.0]);
        // z axis → x axis: the old x (10) moves to -z
        assert!((posed[1][2] + 10.0).abs() < 0.01 && (posed[1][1] + 10.0).abs() < 0.01, "{:?}", posed[1]);
    }
}
