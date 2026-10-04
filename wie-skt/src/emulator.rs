use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format, str,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use encoding_rs::EUC_KR;
use jvm::{Result as JvmResult, runtime::JavaLangString};

use wie_backend::{
    DefaultTaskRunner, Emulator, Event, Pacing, Platform, System,
    canvas::{decode_res, encode_png},
    extract_zip,
};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

pub struct SktEmulator {
    system: System,
}

// `m.MODEL` (read by 59 of the corpus's 84 SKT titles) and `m.EXT_SW` (7) were null; a title that
// calls `.equals` on either unchecked died at boot (2026-09-27 census, one each). The values match
// no real handset or extension, so a title that branches on them takes the default path a
// null-checking one already takes.
const SYSTEM_PROPERTIES: [(&str, &str); 9] = [
    ("MIN", "01000000000"),
    ("m.MIN", "01000000000"),
    ("m.COLOR", "7"),
    ("m.VENDER", "vender"),
    ("m.MODEL", "wie"),
    ("m.EXT_SW", "0"),
    ("m.CARRIER", "SKT"),
    ("m.SK_VM", "10"),
    ("com.xce.wipi.version", ""),
];

// Handset screens an SKT package picks its art for, by width. The heights are the common panel
// for each width — nothing in a package names one.
//   ponytail: one height per width; read it off the package if a title branches on height alone.
const HANDSET_DISPLAYS: [(u32, u32); 4] = [(120, 160), (128, 160), (176, 220), (240, 320)];

/// The display an SKT package was built for, when its contents say so. Unlike KTF's `DisplaySize:`
/// nothing declares it, but the code does: it names `<stem>_<width>.<ext>` for several widths and
/// picks one by the reported screen width, and a package cut for small handsets ships only the
/// small ones. bf54c05e58a9 names `_240` and holds only `_120`/`_176`, so on a 240-wide screen it
/// draws a null image and stops; 78bd51675574 holds only `_120` of three. The widest width held
/// beside a missing one is the screen it was cut for. Corpus scan 2026-10-02: those two titles of
/// 462 packages, both SKT.
fn package_display_size(jar: &[u8]) -> Option<(u32, u32)> {
    let files = extract_zip(jar).ok()?;
    let names = files.keys().map(|name| name.trim_start_matches('/')).collect::<BTreeSet<_>>();
    let references = files
        .iter()
        .filter(|(name, _)| name.ends_with(".class"))
        .flat_map(|(_, class)| class_utf8_constants(class));
    display_size_for(&names, references)
}

fn display_size_for<'a>(names: &BTreeSet<&str>, references: impl Iterator<Item = &'a [u8]>) -> Option<(u32, u32)> {
    let mut held = None;
    for reference in references {
        let Some(path) = str::from_utf8(reference).ok().map(|x| x.trim_start_matches('/')) else {
            continue;
        };
        let Some((stem_width, ext)) = path.rsplit_once('.') else {
            continue;
        };
        let Some((stem, width)) = stem_width.rsplit_once('_') else {
            continue;
        };
        let Some(&(missing, _)) = HANDSET_DISPLAYS.iter().find(|(w, _)| w.to_string() == width) else {
            continue;
        };
        if names.contains(path) {
            continue;
        }
        for &(w, h) in &HANDSET_DISPLAYS {
            if w < missing && names.contains(format!("{stem}_{w}.{ext}").as_str()) && held.is_none_or(|(held_w, _)| w > held_w) {
                held = Some((w, h));
            }
        }
    }
    held
}

/// The CONSTANT_Utf8 strings of a class file's constant pool — where `ldc` string literals live.
fn class_utf8_constants(class: &[u8]) -> Vec<&[u8]> {
    let mut strings = Vec::new();
    let Some(&[high, low]) = class.get(8..10) else {
        return strings;
    };
    let (count, mut at, mut index) = (u16::from_be_bytes([high, low]), 10, 1);
    while index < count {
        let Some(&tag) = class.get(at) else {
            break;
        };
        let length = match tag {
            1 => {
                let Some(&[high, low]) = class.get(at + 1..at + 3) else {
                    break;
                };
                let length = u16::from_be_bytes([high, low]) as usize;
                strings.extend(class.get(at + 3..at + 3 + length));
                2 + length
            }
            7 | 8 | 16 => 2,
            15 => 3,
            3 | 4 | 9 | 10 | 11 | 12 | 18 => 4,
            5 | 6 => 8,
            _ => break,
        };
        at += 1 + length;
        index += if matches!(tag, 5 | 6) { 2 } else { 1 };
    }
    strings
}

impl SktEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> Result<Self> {
        let msd_file = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let msd = SktMsd::parse(msd_file.0, msd_file.1);

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar_filename = msd_file.0.replace(".msd", ".jar");
        if let Some((width, height)) = files.get(&jar_filename).and_then(|jar| package_display_size(jar))
            && width < platform.screen().width()
        {
            tracing::info!("Package holds no assets wider than {width}: display {width}x{height}");
            if let Err(error) = platform.screen().resize(width, height) {
                tracing::warn!("Ignoring unsupported display size {width}x{height}: {error}");
            }
        }

        Self::load(platform, &jar_filename, &msd.id, Some(msd.main_class), msd.properties, &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>, id: &str, main_class_name: Option<String>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, BTreeMap::new(), &files)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn archive_title(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let (filename, data) = files.iter().find(|(filename, _)| filename.ends_with(".msd"))?;
        let title = SktMsd::parse(filename, data).name;
        (!title.is_empty()).then_some(title)
    }

    pub fn archive_id(files: &BTreeMap<String, Vec<u8>>) -> Option<String> {
        let (filename, data) = files.iter().find(|(filename, _)| filename.ends_with(".msd"))?;
        let id = SktMsd::parse(filename, data).id;
        (!id.is_empty()).then_some(id)
    }

    pub fn archive_icon(files: &BTreeMap<String, Vec<u8>>) -> Option<Vec<u8>> {
        if let Some(icon) = files.iter().find(|(filename, _)| filename.ends_with(".wmr")).and_then(|(_, wmr)| {
            if !wmr.starts_with(b"\xad\xde\xce\xfa") {
                return None;
            }

            let icon_size = u32::from_le_bytes(wmr.get(12..16)?.try_into().ok()?) as usize;
            let icon = wmr.get(16..16usize.checked_add(icon_size)?)?;
            icon.starts_with(b"BM").then(|| icon.to_vec())
        }) {
            return Some(icon);
        }

        let (_, resource) = files.iter().find(|(filename, _)| filename.ends_with(".res"))?;
        let image = decode_res(resource).ok()?;
        encode_png(&*image).ok()
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        jar.starts_with(b"\x20\x00\x00\x00\x00\x00\x00\x00")
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        properties: BTreeMap<String, String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let system = System::new(platform, id, id, DefaultTaskRunner);

        for (filename, data) in files {
            system.filesystem().add_virtual(filename, data.clone())
        }

        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();

        system.spawn(async move || Self::do_start(&mut system_clone, jar_filename_clone, properties, main_class_name).await);

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(
        system: &mut System,
        jar_filename: String,
        properties: BTreeMap<String, String>,
        main_class_name: Option<String>,
    ) -> Result<()> {
        let system_properties = SYSTEM_PROPERTIES;
        let properties = properties
            .into_iter()
            .map(|(k, v)| (format!("wie.appProperty.{k}"), v))
            .collect::<Vec<_>>();
        let properties = system_properties
            .into_iter()
            .chain(properties.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .collect::<Vec<_>>();

        let protos = [
            wie_midp::get_protos().into(),
            wie_skvm::get_protos().into(),
            wie_wipi_java::get_protos().into(),
        ];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &properties, RustJavaJvmImplementation).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x.replace('.', "/")
        } else {
            return Err(WieError::FatalError("Main class not found".into()))?;
        };

        let main_class = jvm.resolve_class(&main_class_name).await.unwrap();
        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = if jvm.is_inherited_from(&*main_class.definition, "javax/microedition/midlet/MIDlet") {
            jvm.invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
                .await
        } else {
            let mut args = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            jvm.store_array(&mut args, 0, vec![main_class_java]).await.unwrap();
            jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args,))
                .await
        };

        JvmSupport::finish_launch(&jvm, result).await
    }
}

// Tasks hold clones of the system and the JVM, so dropping the emulator frees
// nothing unless the cycles are cut.
impl Drop for SktEmulator {
    fn drop(&mut self) {
        self.system.teardown();
    }
}

impl Emulator for SktEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn take_pacing(&mut self) -> Pacing {
        self.system.pacing().take()
    }

    fn tick_for(&mut self, budget_ms: u64) -> Result<()> {
        self.system.tick_for(budget_ms)
    }
}

struct SktMsd {
    name: String,
    id: String,
    main_class: String,
    properties: BTreeMap<String, String>,
}

impl SktMsd {
    pub fn parse(filename: &str, data: &[u8]) -> Self {
        let mut name = String::new();
        let mut main_class = String::new();
        let mut id = filename[..filename.find('.').unwrap()].into();
        let mut properties = BTreeMap::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"MIDlet-Name:") {
                name = EUC_KR.decode(&line[12..]).0.trim().to_string();
            } else if line.starts_with(b"MIDlet-1:")
                && let Some(value) = line[9..].split(|x| *x == b',').nth(2)
                && let Ok(value) = str::from_utf8(value)
            {
                main_class = value.trim().to_string();
            }
            if line.starts_with(b"DD-ProgName:")
                && let Ok(value) = str::from_utf8(&line[12..])
            {
                id = value.trim().to_string();
            }

            let sep = line.iter().position(|x| *x == b':');
            if let Some(sep) = sep {
                let key = &line[..sep];
                let value = &line[sep + 1..];

                if let (Ok(key), Ok(value)) = (str::from_utf8(key), str::from_utf8(value)) {
                    tracing::info!("Adding property {}={}", key.trim(), value.trim());
                    properties.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self {
            name,
            id,
            main_class,
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, vec, vec::Vec};

    use super::{SktEmulator, SktMsd};

    #[test]
    fn parse_msd_name() {
        let msd = SktMsd::parse(
            "0051758461.msd",
            b"MIDlet-Name: \xbf\xb5\xbf\xf5\xbc\xad\xb1\xe2_\xbc\xd6\xc6\xbc\xbe\xc6\xc0\xc7\xb9\xd9\xb6\xf7\r\nMIDlet-1: title,,rpg.GameMIDlet\r\nDD-ProgName: 0051758461\r\n",
        );

        assert_eq!(msd.name, "영웅서기_솔티아의바람");
        assert_eq!(msd.id, "0051758461");
        assert_eq!(msd.main_class, "rpg.GameMIDlet");
    }

    #[test]
    fn parse_msd_ignores_malformed_optional_fields() {
        let msd = SktMsd::parse("sample.msd", b"MIDlet-Name: Sample\nMIDlet-1: incomplete\nDD-ProgName: \xff\n");

        assert_eq!(msd.name, "Sample");
        assert_eq!(msd.id, "sample");
        assert!(msd.main_class.is_empty());
    }

    #[test]
    fn extracts_static_icon_from_wmr() {
        let mut wmr = vec![0; 16];
        wmr[..4].copy_from_slice(b"\xad\xde\xce\xfa");
        wmr[12..16].copy_from_slice(&6u32.to_le_bytes());
        wmr.extend_from_slice(b"BMicon");
        let files = BTreeMap::from([("sample.wmr".into(), wmr)]);

        assert_eq!(SktEmulator::archive_icon(&files), Some(b"BMicon".to_vec()));
    }

    #[test]
    fn extracts_static_icon_from_res_after_animation() {
        let mut resource = vec![0; 15];
        resource[..4].copy_from_slice(b"\xce\xfa\xad\xde");
        resource[6..8].copy_from_slice(&15u16.to_le_bytes());
        resource.extend_from_slice(&[3, 1, 1, 8, 0, 0, 0, 0, 2, 0, 0]);
        resource.extend_from_slice(&[2, 2, 1, 8, 0, 0, 0, 0, 0xe0, 0x03]);
        let files = BTreeMap::from([("sample.res".into(), resource)]);

        let png = SktEmulator::archive_icon(&files).unwrap();
        let image = wie_backend::canvas::decode_image(&png).unwrap();

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 1);
        let red = image.get_pixel(0, 0);
        let blue = image.get_pixel(1, 0);
        assert_eq!((red.r, red.g, red.b, red.a), (252, 0, 0, 255));
        assert_eq!((blue.r, blue.g, blue.b, blue.a), (0, 0, 255, 255));
    }

    // A class whose constant pool is a long, then the strings: the long takes two slots.
    fn class_with(strings: &[&str]) -> Vec<u8> {
        let mut class = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 46];
        class.extend_from_slice(&(strings.len() as u16 + 3).to_be_bytes());
        class.extend_from_slice(&[5, 0, 0, 0, 0, 0, 0, 0, 1]);
        for string in strings {
            class.push(1);
            class.extend_from_slice(&(string.len() as u16).to_be_bytes());
            class.extend_from_slice(string.as_bytes());
        }
        class
    }

    #[test]
    fn display_size_is_the_widest_variant_a_package_holds_beside_a_missing_one() {
        let class = class_with(&["/title/logo_120.png", "/title/logo_176.png", "/title/logo_240.png", "/bg_01.png"]);
        let strings = super::class_utf8_constants(&class);
        assert_eq!(strings.len(), 4, "the long before them takes two slots");

        let held = |names: &[&'static str]| super::display_size_for(&names.iter().copied().collect(), strings.iter().copied());
        assert_eq!(held(&["title/logo_120.png", "title/logo_176.png"]), Some((176, 220)));
        assert_eq!(held(&["title/logo_120.png"]), Some((120, 160)));
        // Every variant held, or none of them: nothing says the package is not for this screen.
        assert_eq!(held(&["title/logo_120.png", "title/logo_176.png", "title/logo_240.png"]), None);
        assert_eq!(held(&[]), None);
    }

    #[test]
    fn model_and_ext_sw_properties_are_set() {
        for key in ["m.MODEL", "m.EXT_SW"] {
            assert!(super::SYSTEM_PROPERTIES.iter().any(|(k, v)| *k == key && !v.is_empty()), "{key}");
        }
    }
}
