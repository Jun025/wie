# WIE

[Homepage](https://wie-site.dlunch.net) | [Try in browser](https://wie.dlunch.net)

A standalone web-based emulator for old mobile apps based on WIPI, SKVM or J2ME.

Our goal is to revive the legacy of classic mobile games and allow them to be experienced in modern web environments.

- [Contribution guide](https://github.com/dlunch/wie/blob/main/CONTRIBUTING.md)
- Architecture docs: [Emulator](docs/architecture.md) | [KTF](docs/ktf.md) | [LGT](docs/lgt.md)

## Web service (this fork)

A browser front-end (`web/`, React + Vite) plus a Cloudflare Pages Functions + D1
backend (`functions/`) for **accounts** and **save sync**. See
[docs/web.md](docs/web.md) and [docs/CLOUDFLARE_SETUP.md](docs/CLOUDFLARE_SETUP.md).

**Privacy (BYOF):** game files are processed **entirely in your browser** — the
bytes, filenames, content hashes, and the "which games this device has" list are
never sent to or stored on the server. Only account info, opaque save data, and
inquiry text reach the server. Enforced by `scripts/audit-no-leak.sh`.

> **Legal note.** The notices and policy text in the app and docs are **not legal
> advice** — operators should have them reviewed by a qualified Korean
> intellectual-property lawyer before running the service publicly. A
> rights-holder report / takedown channel is provided in the app's inquiry page;
> the service does not host or distribute game files (BYOF only), and posting
> download links to infringing material is prohibited.

## Related projects

- [RustJava](https://github.com/dlunch/RustJava)
- [smaf](https://github.com/dlunch/smaf)
