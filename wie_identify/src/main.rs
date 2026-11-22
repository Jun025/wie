//! `wie_identify <file>…` — one line per file, in argument order: `contentKey<TAB>platform<TAB>model`,
//! `-` for a part the file does not have. Prints nothing else about the file (no name, no bytes):
//! `scripts/content-key.mjs` pairs the lines with the paths it passed.

fn main() -> std::io::Result<()> {
    for path in std::env::args().skip(1) {
        let identity = wie_identify::identify(&path, &std::fs::read(&path)?);
        let (key, platform, model) = match &identity {
            Some(x) => (x.content_key.as_deref(), Some(x.platform), x.model),
            None => (None, None, None),
        };
        println!("{}\t{}\t{}", key.unwrap_or("-"), platform.unwrap_or("-"), model.unwrap_or("-"));
    }
    Ok(())
}
