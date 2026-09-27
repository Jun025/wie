use alloc::{vec, vec::Vec};

use encoding_rs::EUC_KR;
use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.io.ByteToCharEUC_KR
//
// convert(in, inOff, inLen, out, outOff, outLen) takes LENGTHS, not end indices: the one caller
// measured (fb80e97cbc57) passes `end - start` and `out.length - outOff` (bytecode read 2026-09-27).
// Returns the number of chars written.
// ponytail: stateless — a two-byte character split across two convert calls decodes as U+FFFD
// halves; keep a pending lead byte in a field if a game streams text in small chunks.
pub struct ByteToCharEucKr;

impl ByteToCharEucKr {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/ByteToCharEUC_KR",
            parent_class: Some("com/xce/io/ByteToCharConverter"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("convert", "([BII[CII)I", Self::convert, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("flush", "([CII)I", Self::flush, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "com/xce/io/ByteToCharConverter", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn convert(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        input: ClassInstanceRef<Array<i8>>,
        in_offset: i32,
        in_length: i32,
        mut output: ClassInstanceRef<Array<JavaChar>>,
        out_offset: i32,
        out_length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::convert({this:?}, {in_offset}, {in_length}, {out_offset}, {out_length})");

        let bytes: Vec<i8> = jvm.load_array(&input, in_offset as _, in_length.max(0) as _).await?;
        let bytes: Vec<u8> = bytes.into_iter().map(|x| x as u8).collect();
        let (text, _, _) = EUC_KR.decode(&bytes);
        let chars: Vec<JavaChar> = text.encode_utf16().take(out_length.max(0) as usize).collect();
        let written = chars.len() as i32;
        jvm.store_array(&mut output, out_offset as _, chars).await?;

        Ok(written)
    }

    async fn flush(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, _: ClassInstanceRef<Array<JavaChar>>, _: i32, _: i32) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.ByteToCharEUC_KR::flush({this:?})");

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{JavaChar, Result as JvmResult};
    use test_utils::run_jvm_test;

    use crate::get_protos;

    /// fb80e97cbc57 decodes its Korean text through this converter at boot; without the class it
    /// died on `NoClassDefFoundError: com/xce/io/ByteToCharEUC_KR`. "가A" is EUC-KR B0 A1 41, and
    /// the length arguments (not end indices) must be honoured on both sides.
    #[test]
    fn euc_kr_convert_decodes_with_length_arguments() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into()]),
            |jvm| async move {
                let converter = jvm.new_class("com/xce/io/ByteToCharEUC_KR", "()V", ()).await?;
                let mut input = jvm.instantiate_array("B", 5).await?;
                jvm.store_array(&mut input, 0, [0x7f_i8, 0xb0_u8 as i8, 0xa1_u8 as i8, 0x41, 0x7f]).await?;
                let output = jvm.instantiate_array("C", 4).await?;

                let written: i32 = jvm
                    .invoke_virtual(&converter, "com/xce/io/ByteToCharConverter", "convert", "([BII[CII)I", (input, 1, 3, output.clone(), 1, 3))
                    .await?;
                assert_eq!(written, 2);
                let chars: Vec<JavaChar> = jvm.load_array(&output, 0, 4).await?;
                assert_eq!(chars, [0, 0xac00, 0x41, 0]);

                let flushed: i32 = jvm
                    .invoke_virtual(&converter, "com/xce/io/ByteToCharConverter", "flush", "([CII)I", (output, 0, 4))
                    .await?;
                assert_eq!(flushed, 0);

                JvmResult::Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");
    }
}
