# Third-party audio components of the featurephone engine artifact

The engine artifact (`wie_web.js` + `wie_web_bg.wasm`) carries a soundfont synthesizer in the audio
worklet, and the host serves a soundfont for it. Both come from third parties. This directory is
what travels with them: `publish-artifact.yml` attaches these files to every engine release, next
to the two artifact files, so the shell's pin bump receives them together.

| component | version | where it ends up | license | obligation |
|---|---|---|---|---|
| spessasynth_core | 4.3.18 (exact pin, `package.json`) | bundled into `wie_web_bg.wasm` (the soundfont prelude) | Apache License 2.0 — `Apache-2.0.txt` | §4(a): give recipients a copy of the license. The prelude itself also opens with a one-line notice naming this file. No `NOTICE` file exists upstream. |
| stb-vorbis | 0.0.6 (spessasynth_core's dependency) | same | Apache License 2.0 — same text, byte-identical `LICENSE` | same |
| GeneralUser GS | 2.0.1 | NOT in the artifact — the host serves the `.sf3` and passes its URL to `new WieEmulator(…, soundfontUrl)` | GeneralUser GS License v2.0 — `GeneralUser-GS-2.0.1-LICENSE.txt` | Use and redistribution permitted; "provide your own local copy" (no hot-linking the author's downloads). The text disclaims certainty about every sample's origin — it does not forbid commercial use. |

Decision record: `docs/report/0317` (license texts quoted from the npm tarballs and the sf3), and the
round that shipped this: see `docs/worklog/2026-09-28-featurephone-soundfont-lazy-load.json`.
