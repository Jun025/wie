# License-clean MIDP/WIPI corpus

Games whose redistribution is permitted **in writing**, collected to find where the engine falls
short on general MIDP titles. The carrier corpus (`game_lab/`, git-ignored, Constraint 9) is mostly
WIPI-C and carrier Java; it almost never reaches the MIDP 2.0 surface a general J2ME game is built
on (`lcdui.game`, `List`, MIDI/WAV players). This corpus does. Round and measurements:
`docs/report/0483`.

## What is here, and what is not

| file | what |
|---|---|
| `candidates.csv` | every candidate looked at (78): source URL, license, the evidence for it (file path or quoted header), commit, build, verdict (채택 / 보류 / 제외) and the reason |
| `agneay-100-games.csv` | the 100 titles inside the one adopted collection (`github.com/agneay/…`, MIT), with their MIDlet class and the sha256 of each rebuilt jar |

**No game jar is committed.** Three reasons, any one of which would be enough:

1. Constraint 9 ("no game bytes, ever") is written about the carrier corpus, but the mechanism it
   protects — the repo publishes a WASM artifact that otterpebble deploys — does not know the
   difference between a licensed jar and an unlicensed one. Keeping all game bytes out keeps the
   rule checkable.
2. Ten of the adopted titles are GPL. Shipping a GPL binary obliges the shipper to offer the
   corresponding source; a jar in this repo would put that obligation on every artifact built from it.
3. Nothing needs them here: every jar is rebuilt from a pinned upstream commit, so the ledger
   (URL + commit + build + sha256) reproduces it.

The jars, the verbatim license texts and the build scratch live outside the repo, in the lab
directory the round used (`~/work/oss-j2me-lab/`: `jars/`, `jars-wipi/`, `licenses/`, `src/`,
`build/build1.sh`).

## Rules the verdicts follow

- **Adopt** only on a written license: an OSI license file or per-file headers, CC0/public domain
  stated, or a free-redistribution grant from the author quoted with its URL. No license = 제외.
- **Hold** when the code is licensed but something it needs is not: assets with no stated source,
  a third party's IP (title, characters, music), a non-commercial (NC) license, a bundled
  proprietary SDK. Holding is not a judgement on the author — it is "not provable from the repo".
- **Exclude** decompilations, ports of commercial games, clones named after a commercial title,
  and anything whose assets were lifted from firmware.
- A file inside an adopted repo whose header points elsewhere is replaced rather than guessed at:
  `j2me-lines`' `util/ImageHelper.java` names a Nokia `LICENSE.TXT` the repository does not ship,
  so the lab build uses a clean-room 15-line equivalent (same signature, GPL-3.0 like the rest).

## If a title is ever served (the shell's 체험 게임)

- **GPL** (j2me-lines, slime-volleyball, kurve, sperm-race, bubblet-asha, minitruco, j2me-2048,
  pipes, mobapp-game): serve the source alongside — the upstream URL at the recorded commit plus any
  lab patch — and show the license.
- **MIT / BSD-3 / MIT-0 / CC0**: show the copyright line and license text (BSD-3: no endorsement
  using Sun's name for the WTK demos).
- `sperm-race` is adult-themed — never a demo pick.
