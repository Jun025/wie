//! `org.kwis.msp.lwc.ProxyCard` came in with the 2026-11-23 upstream merge. Upstream's own check of
//! its constructors sits inside a ShellComponent test that asserts upstream's shell design (a `cd`
//! field on the shell); ours keeps `net.wie.ShellCard`, so only the ProxyCard part is carried here.

use jvm::ClassInstanceRef;
use jvm_types::ClassAccessFlags;

use test_utils::run_jvm_test;
use wie_jvm_support::WieJavaClassProto;
use wie_util::Result;
use wie_wipi_java::classes::org::kwis::msp::lcdui::Display;

// Apps can wrap a container in a card themselves; every constructor yields a usable card.
#[test]
fn every_proxy_card_constructor_yields_a_usable_card() -> Result<()> {
    let jlet_proto = WieJavaClassProto {
        name: "test/ShellJlet",
        parent_class: Some("org/kwis/msp/lcdui/Jlet"),
        interfaces: vec![],
        methods: vec![],
        fields: vec![],
        access_flags: ClassAccessFlags::PUBLIC,
    };
    run_jvm_test(
        Box::new([wie_midp::get_protos().into(), wie_wipi_java::get_protos().into(), Box::new([jlet_proto])]),
        |jvm| async move {
            let _midlet = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await?;
            let jlet = jvm.instantiate_class("test/ShellJlet").await?;
            let _: () = jvm.invoke_special(&jlet, "org/kwis/msp/lcdui/Jlet", "<init>", "()V", ()).await?;
            let display: ClassInstanceRef<Display> = jvm
                .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", ())
                .await?;
            let display_width: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await?;

            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            let whole_display = jvm
                .new_class("org/kwis/msp/lwc/ProxyCard", "(Lorg/kwis/msp/lwc/ContainerComponent;)V", (shell.clone(),))
                .await?;
            let transparent = jvm
                .new_class(
                    "org/kwis/msp/lwc/ProxyCard",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;Z)V",
                    (shell.clone(), true),
                )
                .await?;
            let bounded = jvm
                .new_class(
                    "org/kwis/msp/lwc/ProxyCard",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;IIII)V",
                    (shell.clone(), 1, 2, 30, 40),
                )
                .await?;
            let bounded_transparent = jvm
                .new_class(
                    "org/kwis/msp/lwc/ProxyCard",
                    "(Lorg/kwis/msp/lwc/ContainerComponent;IIIIZ)V",
                    (shell.clone(), 1, 2, 30, 40, true),
                )
                .await?;
            for (card, expected_width) in [
                (whole_display, display_width),
                (transparent, display_width),
                (bounded, 30),
                (bounded_transparent, 30),
            ] {
                let width: i32 = jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "getWidth", "()I", ()).await?;
                assert_eq!(width, expected_width);
            }

            Ok(())
        },
    )
}
