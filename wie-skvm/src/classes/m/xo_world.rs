use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::io::InputStream;

use wie_backend::canvas::{ArgbPixel, Clip, VecImageBuffer};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

use super::{A3, V3};
use crate::mascot;

// class m.XO_World — SKT's micro3D v3 figure: one MBAC model, one BMP texture, one MTRA action
// table, drawn with a parallel projection set by `setView`.
pub struct XOWorld;

const MODEL: &str = "model";
const TEXTURE: &str = "texture";
const ACTIONS: &str = "actions";

impl XOWorld {
    pub fn as_proto() -> WieJavaClassProto {
        let static_public = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        WieJavaClassProto {
            name: "m/XO_World",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loadMBAC", "(Ljava/io/InputStream;)I", Self::load_mbac, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loadBMP", "(Ljava/io/InputStream;)I", Self::load_bmp, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("loadMTRA", "(Ljava/io/InputStream;)I", Self::load_mtra, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("dispose", "()V", Self::dispose, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("shareData", "(Lm/XO_World;)V", Self::share_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setVram",
                    "(Ljavax/microedition/lcdui/Graphics;Lm/XO_World;II)V",
                    Self::set_vram,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setView", "(Lm/A3;IIII)V", Self::set_view, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPosture", "(II)V", Self::set_posture, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMaxFrame", "(I)I", Self::get_max_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("draw", "(Ljavax/microedition/lcdui/Graphics;)V", Self::draw, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getViewTrans", "(Lm/V3;Lm/V3;Lm/V3;Lm/A3;)V", Self::get_view_trans, static_public),
                JavaMethodProto::new("rotY", "(ILm/A3;)V", Self::rot_y, static_public),
                JavaMethodProto::new("rotZ", "(ILm/A3;)V", Self::rot_z, static_public),
                JavaMethodProto::new("sin", "(I)I", Self::sin, static_public),
                JavaMethodProto::new("cos", "(I)I", Self::cos, static_public),
            ],
            fields: vec![
                JavaFieldProto::new(MODEL, "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(TEXTURE, "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new(ACTIONS, "[B", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("action", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frame", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("view", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("clip", "[I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::<init>({this:?})");
        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    // 0 on success, -1 when the data is not a format this loader decodes (logged)
    async fn load(
        jvm: &Jvm,
        mut this: ClassInstanceRef<Self>,
        field: &str,
        stream: ClassInstanceRef<InputStream>,
        valid: fn(&[u8]) -> bool,
    ) -> JvmResult<i32> {
        if stream.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "stream is null").await);
        }
        let data = jvm::runtime::JavaIoInputStream::read_until_end(jvm, &stream).await?;
        if !valid(&data) {
            tracing::warn!(
                "m.XO_World: unsupported {field} data ({} bytes, head {:02x?})",
                data.len(),
                &data[..data.len().min(4)]
            );
            return Ok(-1);
        }
        let mut array = jvm.instantiate_array("B", data.len() as _).await?;
        jvm.array_raw_buffer_mut(&mut array).await?.write(0, &data)?;
        jvm.put_field(&mut this, field, "[B", array).await?;
        Ok(0)
    }

    async fn load_mbac(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        stream: ClassInstanceRef<InputStream>,
    ) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::loadMBAC({this:?})");
        Self::load(jvm, this, MODEL, stream, |d| mascot::parse_mbac(d).is_some()).await
    }

    async fn load_bmp(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        stream: ClassInstanceRef<InputStream>,
    ) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::loadBMP({this:?})");
        Self::load(jvm, this, TEXTURE, stream, |d| mascot::parse_bmp(d).is_some()).await
    }

    async fn load_mtra(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        stream: ClassInstanceRef<InputStream>,
    ) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::loadMTRA({this:?})");
        Self::load(jvm, this, ACTIONS, stream, |d| mascot::parse_mtra(d).is_some()).await
    }

    async fn dispose(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::dispose({this:?})");
        for field in [MODEL, TEXTURE, ACTIONS] {
            jvm.put_field(&mut this, field, "[B", None).await?;
        }
        Ok(())
    }

    async fn share_data(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, other: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::shareData({this:?}, {other:?})");
        if other.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "world is null").await);
        }
        for field in [MODEL, TEXTURE, ACTIONS] {
            let data: ClassInstanceRef<Array<i8>> = jvm.get_field(&other, field, "[B").await?;
            jvm.put_field(&mut this, field, "[B", data).await?;
        }
        Ok(())
    }

    // the render target is the Graphics passed to `draw`; the size given here only sizes the
    // phone's own frame buffer, which this renderer does not need
    async fn set_vram(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        share: ClassInstanceRef<Self>,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("m.XO_World::setVram({this:?}, {graphics:?}, {share:?}, {width}, {height})");
        Ok(())
    }

    async fn set_clip(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32) -> JvmResult<()> {
        tracing::debug!("m.XO_World::setClip({this:?}, {x}, {y}, {w}, {h})");
        let mut clip = jvm.instantiate_array("I", 4).await?;
        jvm.store_array(&mut clip, 0, [x, y, w, h]).await?;
        jvm.put_field(&mut this, "clip", "[I", clip).await
    }

    // parallel projection: screen = (cx, cy) + (view · v) · (sx, sy) / 4096
    #[allow(clippy::too_many_arguments)]
    async fn set_view(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        view: ClassInstanceRef<A3>,
        sx: i32,
        sy: i32,
        cx: i32,
        cy: i32,
    ) -> JvmResult<()> {
        tracing::debug!("m.XO_World::setView({this:?}, {view:?}, {sx}, {sy}, {cx}, {cy})");
        let m = A3::get(jvm, &view).await?;
        let mut array = jvm.instantiate_array("I", 16).await?;
        jvm.store_array(&mut array, 0, m.into_iter().chain([sx, sy, cx, cy])).await?;
        jvm.put_field(&mut this, "view", "[I", array).await
    }

    // frame is 16.16 fixed point, like getMaxFrame's result
    async fn set_posture(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, action: i32, frame: i32) -> JvmResult<()> {
        tracing::debug!("m.XO_World::setPosture({this:?}, {action}, {frame})");
        jvm.put_field(&mut this, "action", "I", action).await?;
        jvm.put_field(&mut this, "frame", "I", frame).await
    }

    async fn get_max_frame(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, action: i32) -> JvmResult<i32> {
        tracing::debug!("m.XO_World::getMaxFrame({this:?}, {action})");
        let keyframes = Self::bytes(jvm, &this, ACTIONS)
            .await?
            .and_then(|d| mascot::parse_mtra(&d))
            .and_then(|a| a.keyframes(usize::try_from(action).ok()?));
        Ok(keyframes.map_or(0, |k| (k as i32) << 16))
    }

    async fn draw(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, mut graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("m.XO_World::draw({this:?}, {graphics:?})");
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }
        let Some(model) = Self::bytes(jvm, &this, MODEL).await?.and_then(|d| mascot::parse_mbac(&d)) else {
            tracing::warn!("m.XO_World::draw without a model");
            return Ok(());
        };
        let view: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "view", "[I").await?;
        if view.is_null() {
            tracing::warn!("m.XO_World::draw before setView");
            return Ok(());
        }
        let view = jvm.load_array::<i32>(&view, 0, 16).await?;
        let texture = Self::bytes(jvm, &this, TEXTURE).await?.and_then(|d| mascot::parse_bmp(&d));
        let actions = Self::bytes(jvm, &this, ACTIONS).await?.and_then(|d| mascot::parse_mtra(&d));
        let action: i32 = jvm.get_field(&this, "action", "I").await?;
        let frame: i32 = jvm.get_field(&this, "frame", "I").await?;

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let mut clip = Graphics::clip(jvm, &graphics).await?;
        let world_clip: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "clip", "[I").await?;
        if !world_clip.is_null() {
            let c = jvm.load_array::<i32>(&world_clip, 0, 4).await?;
            clip = clip.intersect(&Clip {
                x: c[0].wrapping_add(translate_x),
                y: c[1].wrapping_add(translate_y),
                width: c[2].max(0) as u32,
                height: c[3].max(0) as u32,
            });
        }
        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        let bounds = Clip {
            x: 0,
            y: 0,
            width: canvas.image().width(),
            height: canvas.image().height(),
        };
        let clip = clip.intersect(&bounds);

        let vertices = mascot::pose(&model, actions.as_ref().and_then(|a| Some((a, usize::try_from(action).ok()?, frame))));
        let matrix: mascot::Affine = view[..12].try_into().unwrap();
        let Some(rendered) = mascot::render(
            &model,
            &vertices,
            texture.as_ref(),
            &matrix,
            (view[12], view[13]),
            (view[14].wrapping_add(translate_x), view[15].wrapping_add(translate_y)),
            (clip.x, clip.y, clip.width as i32, clip.height as i32),
        ) else {
            return Ok(());
        };
        let (width, height) = (rendered.width as u32, rendered.height as u32);
        let buffer = VecImageBuffer::<ArgbPixel>::from_raw(width, height, rendered.pixels);
        canvas.draw(rendered.x, rendered.y, width, height, &buffer, 0, 0, clip);

        Ok(())
    }

    async fn get_view_trans(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        pos: ClassInstanceRef<V3>,
        look: ClassInstanceRef<V3>,
        up: ClassInstanceRef<V3>,
        out: ClassInstanceRef<A3>,
    ) -> JvmResult<()> {
        let m = mascot::look_at(V3::get(jvm, &pos).await?, V3::get(jvm, &look).await?, V3::get(jvm, &up).await?);
        tracing::debug!("m.XO_World::getViewTrans → {m:?}");
        A3::set(jvm, out, &m).await
    }

    async fn rot_y(jvm: &Jvm, _context: &mut WieJvmContext, angle: i32, out: ClassInstanceRef<A3>) -> JvmResult<()> {
        let mut m = A3::get(jvm, &out).await?;
        mascot::rotation_y(&mut m, angle);
        A3::set(jvm, out, &m).await
    }

    async fn rot_z(jvm: &Jvm, _context: &mut WieJvmContext, angle: i32, out: ClassInstanceRef<A3>) -> JvmResult<()> {
        let mut m = A3::get(jvm, &out).await?;
        mascot::rotation_z(&mut m, angle);
        A3::set(jvm, out, &m).await
    }

    async fn sin(_jvm: &Jvm, _context: &mut WieJvmContext, angle: i32) -> JvmResult<i32> {
        Ok(mascot::isin(angle))
    }

    async fn cos(_jvm: &Jvm, _context: &mut WieJvmContext, angle: i32) -> JvmResult<i32> {
        Ok(mascot::icos(angle))
    }

    async fn bytes(jvm: &Jvm, this: &ClassInstanceRef<Self>, field: &str) -> JvmResult<Option<Vec<u8>>> {
        let array: ClassInstanceRef<Array<i8>> = jvm.get_field(this, field, "[B").await?;
        if array.is_null() {
            return Ok(None);
        }
        let len = jvm.array_length(&array).await?;
        let data: Vec<i8> = jvm.load_array(&array, 0, len).await?;
        Ok(Some(data.into_iter().map(|b| b as u8).collect()))
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, Result as JvmResult};
    use rustjava_runtime::classes::java::io::InputStream;
    use test_utils::run_jvm_test;

    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

    use super::super::{A3, V3, XOWorld};
    use crate::mascot;

    async fn stream(jvm: &jvm::Jvm, data: &[u8]) -> JvmResult<ClassInstanceRef<InputStream>> {
        let mut array = jvm.instantiate_array("B", data.len()).await?;
        jvm.array_raw_buffer_mut(&mut array).await?.write(0, data)?;
        Ok(jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (array,)).await?.into())
    }

    #[test]
    fn world_draws_its_model_through_the_view_and_the_static_helpers_match_micro3d() {
        run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [A3::as_proto(), V3::as_proto(), XOWorld::as_proto()].into(),
            ]),
            |jvm| async move {
                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (40, 40),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &target,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;

                let world: ClassInstanceRef<XOWorld> = jvm.new_class("m/XO_World", "()V", ()).await?.into();
                let model = stream(&jvm, &mascot::quad_model(false)).await?;
                let loaded: i32 = jvm
                    .invoke_virtual(&world, "m/XO_World", "loadMBAC", "(Ljava/io/InputStream;)I", (model,))
                    .await?;
                assert_eq!(loaded, 0);
                let junk = stream(&jvm, b"not a model").await?;
                let rejected: i32 = jvm
                    .invoke_virtual(&world, "m/XO_World", "loadMBAC", "(Ljava/io/InputStream;)I", (junk,))
                    .await?;
                assert_eq!(rejected, -1);
                let frames: i32 = jvm.invoke_virtual(&world, "m/XO_World", "getMaxFrame", "(I)I", (0,)).await?;
                assert_eq!(frames, 0); // no MTRA loaded

                // the quad is 20×20 around the origin; a 2× view centred at (20, 20) covers 0..40
                let view: ClassInstanceRef<A3> = jvm.new_class("m/A3", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(&world, "m/XO_World", "setView", "(Lm/A3;IIII)V", (view.clone(), 8192, 4096, 20, 20))
                    .await?;
                let _: () = jvm.invoke_virtual(&world, "m/XO_World", "setClip", "(IIII)V", (0, 0, 30, 40)).await?;
                let _: () = jvm
                    .invoke_virtual(&world, "m/XO_World", "draw", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
                    .await?;
                {
                    let canvas = Image::canvas(&jvm, &target).await?;
                    let grey = |x, y| canvas.get_pixel(x, y).map(|c| (c.r, c.g, c.b));
                    assert_eq!(grey(2, 20), Some((0x80, 0x80, 0x80))); // x scale 2: −10 → 0
                    assert_eq!(grey(20, 12), Some((0x80, 0x80, 0x80)));
                    assert_eq!(grey(20, 5), Some((0, 0, 0))); // y scale 1: −10 → 10
                    assert_eq!(grey(35, 20), Some((0, 0, 0))); // world clip ends at 30
                }

                // getViewTrans = micro3D lookAt; rotY sets the rotation part; trans = point transform
                let pos: ClassInstanceRef<V3> = jvm.new_class("m/V3", "(III)V", (0, 0, -200)).await?.into();
                let look: ClassInstanceRef<V3> = jvm.new_class("m/V3", "(III)V", (0, 0, 200)).await?.into();
                let up: ClassInstanceRef<V3> = jvm.new_class("m/V3", "(III)V", (0, 4096, 0)).await?.into();
                let _: () = jvm
                    .invoke_static(
                        "m/XO_World",
                        "getViewTrans",
                        "(Lm/V3;Lm/V3;Lm/V3;Lm/A3;)V",
                        (pos.clone(), look, up, view.clone()),
                    )
                    .await?;
                let out: ClassInstanceRef<V3> = jvm.new_class("m/V3", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(&view, "m/A3", "trans", "(Lm/V3;Lm/V3;)V", (pos.clone(), out.clone()))
                    .await?;
                assert_eq!(V3::get(&jvm, &out).await?, [0, 0, 0]); // the camera sits at the view origin
                let _: () = jvm.invoke_static("m/XO_World", "rotY", "(ILm/A3;)V", (1024, view.clone())).await?;
                assert_eq!(&A3::get(&jvm, &view).await?[..3], &[0, 0, 4096]);
                let sin: i32 = jvm.invoke_static("m/XO_World", "sin", "(I)I", (1024,)).await?;
                assert_eq!(sin, 4096);

                Ok(())
            },
        )
        .unwrap();
    }
}
