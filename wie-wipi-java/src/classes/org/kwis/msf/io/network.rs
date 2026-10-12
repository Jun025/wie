use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.Network
pub struct Network;

impl Network {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Network",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "connect",
                    "()I",
                    Self::connect,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "disconnect",
                    "()V",
                    Self::disconnect,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // 0, not -1: the failure belongs to opening the socket (`URL.find` throws SchemeNotFoundException),
    // which titles catch and report. A title that reads -1 as «no stream, carry on» wrote to a null
    // stream and lost its network thread instead (docs/report/0511 §3); census pair over the 160
    // titles shipping this class: docs/report/0514.
    async fn connect(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msf.io.Network::connect()");

        Ok(0)
    }

    async fn disconnect(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn connect_succeeds_so_the_socket_reports_the_failure() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let rc: i32 = jvm.invoke_static("org/kwis/msf/io/Network", "connect", "()I", ()).await?;
            assert_eq!(rc, 0, "connect() must succeed; URL.find is where networking fails");

            Ok(())
        })
    }
}
