use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    javax::microedition::lcdui::{Graphics, Image},
    net::wie::{KeyboardEventType, MIDPKeyCode},
};

// class javax.microedition.lcdui.game.GameCanvas
pub struct GameCanvas;

impl GameCanvas {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/GameCanvas",
            parent_class: Some("javax/microedition/lcdui/Canvas"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "getGraphics",
                    "()Ljavax/microedition/lcdui/Graphics;",
                    Self::get_graphics,
                    MethodAccessFlags::PROTECTED,
                ),
                JavaMethodProto::new("flushGraphics", "()V", Self::flush_graphics, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("flushGraphics", "(IIII)V", Self::flush_graphics_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getKeyStates", "()I", Self::get_key_states, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("handleKeyEvent", "(II)V", Self::handle_key_event, MethodAccessFlags::empty()),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    Self::paint,
                    MethodAccessFlags::PROTECTED,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("offscreenImage", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                // keys held now, and keys pressed since the last getKeyStates (so a tap between two polls is not lost)
                JavaFieldProto::new("keysHeld", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("keysLatched", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, suppress_key_events: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.GameCanvas::<init>({this:?}, {suppress_key_events})");

        let _: () = jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await?;

        let width: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Displayable", "getHeight", "()I", ())
            .await?;

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        jvm.put_field(&mut this, "offscreenImage", "Ljavax/microedition/lcdui/Image;", image)
            .await?;

        Ok(())
    }

    async fn get_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Graphics>> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::getGraphics({this:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "offscreenImage", "Ljavax/microedition/lcdui/Image;").await?;
        let graphics = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Image;)V",
                (offscreen_image,),
            )
            .await?;

        Ok(graphics.into())
    }

    async fn flush_graphics(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::flushGraphics({this:?})");

        let _: () = jvm.invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "()V", ()).await?;

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::paint({this:?}, {g:?})");

        let offscreen_image: ClassInstanceRef<Image> = jvm.get_field(&this, "offscreenImage", "Ljavax/microedition/lcdui/Image;").await?;

        let _: () = jvm
            .invoke_virtual(
                &g,
                "javax/microedition/lcdui/Graphics",
                "drawImage",
                "(Ljavax/microedition/lcdui/Image;III)V",
                (offscreen_image, 0, 0, 0),
            )
            .await?;

        Ok(())
    }

    async fn flush_graphics_area(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.game.GameCanvas::flushGraphics({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_virtual(&this, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (x, y, width, height))
            .await?;

        Ok(())
    }

    async fn get_key_states(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let held: i32 = jvm.get_field(&this, "keysHeld", "I").await?;
        let latched: i32 = jvm.get_field(&this, "keysLatched", "I").await?;
        jvm.put_field(&mut this, "keysLatched", "I", 0).await?;

        Ok(held | latched)
    }

    // Tracks the game keys for getKeyStates, then delivers the event as Canvas does.
    // ponytail: suppressKeyEvents is not honoured — game keys still reach keyPressed. A canvas that asks
    // for suppression polls instead, so the extra call is a no-op unless it also overrides keyPressed.
    async fn handle_key_event(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, event_type: i32, code: i32) -> JvmResult<()> {
        let bit = game_key_bit(code);
        if bit != 0 {
            let held: i32 = jvm.get_field(&this, "keysHeld", "I").await?;
            match KeyboardEventType::from_raw(event_type) {
                Some(KeyboardEventType::KeyPressed) => {
                    let latched: i32 = jvm.get_field(&this, "keysLatched", "I").await?;
                    jvm.put_field(&mut this, "keysHeld", "I", held | bit).await?;
                    jvm.put_field(&mut this, "keysLatched", "I", latched | bit).await?;
                }
                Some(KeyboardEventType::KeyReleased) => jvm.put_field(&mut this, "keysHeld", "I", held & !bit).await?,
                _ => {}
            }
        }

        jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "handleKeyEvent", "(II)V", (event_type, code))
            .await
    }
}

// GameCanvas.*_PRESSED bit for a key: 1 << the key's game action. The number keys stand in for the
// d-pad as on most handsets (2/4/6/8, 5 = fire), so a keypad-only player can still steer.
fn game_key_bit(code: i32) -> i32 {
    let action = match MIDPKeyCode::from_raw(code) {
        Some(MIDPKeyCode::UP | MIDPKeyCode::KEY_NUM2) => 1,
        Some(MIDPKeyCode::LEFT | MIDPKeyCode::KEY_NUM4) => 2,
        Some(MIDPKeyCode::RIGHT | MIDPKeyCode::KEY_NUM6) => 5,
        Some(MIDPKeyCode::DOWN | MIDPKeyCode::KEY_NUM8) => 6,
        Some(MIDPKeyCode::FIRE | MIDPKeyCode::KEY_NUM5) => 8,
        Some(MIDPKeyCode::KEY_NUM7) => 9,   // GAME_A
        Some(MIDPKeyCode::KEY_NUM9) => 10,  // GAME_B
        Some(MIDPKeyCode::KEY_STAR) => 11,  // GAME_C
        Some(MIDPKeyCode::KEY_POUND) => 12, // GAME_D
        _ => return 0,
    };

    1 << action
}
