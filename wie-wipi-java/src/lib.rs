#![no_std]

extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

// `java/io/InterruptedIOException`, `java/io/UnsupportedEncodingException`,
// `java/lang/VirtualMachineError` and `java/lang/OutOfMemoryError` are NOT registered here.
// They were, from 2026-07-02 (`5603a7f9`), because the Jun025/RustJava fork pinned at the time
// did not carry them and KTF's `MExe_init` aborts the guest if a preloaded class does not
// resolve. The pin moved to `dlunch/RustJava@5b84dd1` on 2026-09-04 (`1762a32c`), which
// registers all four, and `JvmSupport::new_jvm` builds `java.class.path` as
// `RT_RUSTJAR : WIE_RUSTJAR : <jar>` — runtime first, so the copies here were unreachable.
// Measured, not reasoned: with them still registered, `new java/lang/VirtualMachineError()`
// already threw `InstantiationError`, which only the runtime's ABSTRACT definition does.
// Locked by `tests/preload_classes_come_from_the_runtime.rs`.
pub fn get_protos() -> [WieJavaClassProto; 51] {
    [
        crate::classes::org::kwis::msp::lcdui::ImageObserver::as_proto(),
        crate::classes::org::kwis::msp::lcdui::InputMethodListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::ActionListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::FormComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::GrabKeyListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::LabelComponent::as_proto(),
        crate::classes::org::kwis::msp::media::MediaUnsupportedException::as_proto(),
        crate::classes::org::kwis::msf::io::Message::as_proto(),
        crate::classes::org::kwis::msf::io::Network::as_proto(),
        crate::classes::org::kwis::msf::io::SchemeNotFoundException::as_proto(),
        crate::classes::org::kwis::msf::io::Socket::as_proto(),
        crate::classes::org::kwis::msf::io::URL::as_proto(),
        crate::classes::org::kwis::msp::db::DataBase::as_proto(),
        crate::classes::org::kwis::msp::db::DataComparator::as_proto(),
        crate::classes::org::kwis::msp::db::DataFilter::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseException::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        crate::classes::org::kwis::msp::handset::BackLight::as_proto(),
        crate::classes::org::kwis::msp::handset::HandsetProperty::as_proto(),
        crate::classes::org::kwis::msp::io::File::as_proto(),
        crate::classes::org::kwis::msp::io::FileSystem::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Card::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Display::as_proto(),
        crate::classes::org::kwis::msp::lcdui::EventQueue::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Font::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Graphics::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Image::as_proto(),
        crate::classes::org::kwis::msp::lcdui::ImageObserver::as_proto(),
        crate::classes::org::kwis::msp::lcdui::InputMethodHandler::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Main::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Jlet::as_proto(),
        crate::classes::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        crate::classes::org::kwis::msp::lcdui::JletWrapper::as_proto(),
        crate::classes::org::kwis::msp::lwc::Component::as_proto(),
        crate::classes::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::DialogComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::EventListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::ShellComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextBoxComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        crate::classes::org::kwis::msp::media::BaseClip::as_proto(),
        crate::classes::org::kwis::msp::media::Clip::as_proto(),
        crate::classes::org::kwis::msp::media::Player::as_proto(),
        crate::classes::org::kwis::msp::media::PlayListener::as_proto(),
        crate::classes::org::kwis::msp::media::Vibrator::as_proto(),
        crate::classes::org::kwis::msp::media::Volume::as_proto(),
        crate::classes::net::wie::CardCanvas::as_proto(),
        crate::classes::net::wie::WIPIFileOutputStream::as_proto(),
        crate::classes::net::wie::WIPIMIDlet::as_proto(),
    ]
}
