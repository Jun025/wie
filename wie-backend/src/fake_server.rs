//! Stand-ins for game servers that no longer exist, run inside the engine.
//!
//! Nothing here opens a host socket, in the native host or in the browser: a guest connect to a
//! listed (title, host, port) reaches one of the responders below, and every other connect fails
//! exactly as it did before this module existed. A title is listed by its `AID` (the carrier's
//! content id from `app_info`, unique across the 91 LGT files of the corpus) — a connect is only
//! answered for the title whose server it was, so a second title that happens to dial the same host
//! is not handed a reply written for someone else.
//!
//! A responder sees the bytes the guest has written so far on one connection and answers with the
//! bytes the guest will read. The replies are the smallest ones the title's own parser accepts as
//! «success»; the real server's shape is unknown. docs/report/0542 has, per title, the parser read
//! off the image and what each reply byte satisfies.

use alloc::{
    collections::{BTreeMap, VecDeque},
    string::String,
    vec,
    vec::Vec,
};

type Respond = fn(&mut Vec<u8>) -> Option<Vec<u8>>;

/// One listed server: whose it is, where the title dials it, and what it answers.
struct Server {
    aid: &'static str,
    host: [u8; 4],
    port: u16,
    respond: Respond,
}

const SERVERS: &[Server] = &[
    // caf9d76ffd13: the first-run «인증» asks one login and nothing else before the title screen.
    // Port 31106 or 30806 by a mode byte (binary.mod 0x14bdc); both are the same login.
    Server {
        aid: "00026E9C",
        host: [218, 145, 70, 36],
        port: 31106,
        respond: login_0102,
    },
    Server {
        aid: "00026E9C",
        host: [218, 145, 70, 36],
        port: 30806,
        respond: login_0102,
    },
    // b9bfcaf42722: the first-run certificate download — a hello, then the certificate itself.
    Server {
        aid: "0002A61E",
        host: [211, 115, 66, 250],
        port: 15133,
        respond: certificate_c2s,
    },
];

/// caf9d76ffd13's packets: `[total length u16 LE][command u16 LE][body]`, the length counting the
/// four header bytes (binary.mod 0x19c3c reads the header, then `length - 4` more). A reply's body
/// starts with a result byte the dispatcher stores before it switches on the command (0x17c58).
///
/// Login `0x0102` (phone number, model, version — built at 0x14c90) is answered with `0x0103`:
/// result 1 (the only value its handler at 0x17a88 goes on with), the 40-byte account record it
/// writes to `audio.adt` (0x164cc; the load at 0x16674 checks only the phone number saved beside
/// it), and a last byte 1. With 0 there the title asks a second, optional question (SMS 수신 동의)
/// that dials again; 1 is «done» and shows «인증이 완료되었습니다». Any other command gets no answer.
fn login_0102(request: &mut Vec<u8>) -> Option<Vec<u8>> {
    let packet = take_packet(request, |x| u16::from_le_bytes([x[0], x[1]]) as usize)?;
    if packet[2..4] != [0x02, 0x01] {
        return None;
    }
    let mut reply = vec![0, 0, 0x03, 0x01, 1];
    reply.extend_from_slice(&[0; 40]);
    reply.push(1);
    reply[0] = reply.len() as u8;

    Some(reply)
}

/// b9bfcaf42722's packets: `[total length u16 BE][0][type][body]` (binary.mod 0x498ce reads the two
/// length bytes, then `length - 2` more). Its certificate module (handler 0x1e4a8) dials, sends a
/// hello (type 0) and, on the reply of the same type, asks for the certificate (type 0x14); the
/// reply to that carries `[u16 BE length][bytes]` from offset 6, which the title writes to
/// `cert.c2s` (0x1e596, 0x1e19c). Sixteen zero bytes were enough: the relaunched title went
/// straight to its story without asking again (docs/report/0542).
fn certificate_c2s(request: &mut Vec<u8>) -> Option<Vec<u8>> {
    let packet = take_packet(request, |x| u16::from_be_bytes([x[0], x[1]]) as usize)?;
    match packet[3] {
        0x00 => Some(vec![0, 6, 0, 0x00, 0, 0]),
        0x14 => {
            let mut reply = vec![0, 0, 0, 0x14, 0, 0, 0, 16];
            reply.extend_from_slice(&[0; 16]);
            reply[1] = reply.len() as u8;
            Some(reply)
        }
        _ => None,
    }
}

/// The first whole packet in `request`, removed from it; None until one has arrived. `length` reads
/// the packet's total length from its first bytes.
fn take_packet(request: &mut Vec<u8>, length: fn(&[u8]) -> usize) -> Option<Vec<u8>> {
    if request.len() < 4 {
        return None;
    }
    let total = length(request).max(4);
    if request.len() < total {
        return None;
    }

    Some(request.drain(..total).collect())
}

pub struct FakeNet {
    aid: String,
    sockets: BTreeMap<i32, FakeSocket>,
    next_fd: i32,
}

struct FakeSocket {
    respond: Option<Respond>,
    request: Vec<u8>,
    reply: VecDeque<u8>,
    // `(cb, param)` a guest left waiting for the reply.
    read_cb: Option<(u32, u32)>,
}

impl FakeNet {
    pub fn new(aid: &str) -> Self {
        Self {
            aid: aid.into(),
            sockets: BTreeMap::new(),
            next_fd: 0,
        }
    }

    /// Whether this title has a listed server at all — the gate for every call below, so a title
    /// with none sees the network exactly as before.
    pub fn online(&self) -> bool {
        SERVERS.iter().any(|x| x.aid == self.aid)
    }

    pub fn socket(&mut self) -> Option<i32> {
        if !self.online() {
            return None;
        }
        let fd = self.next_fd;
        self.next_fd += 1;
        self.sockets.insert(
            fd,
            FakeSocket {
                respond: None,
                request: Vec::new(),
                reply: VecDeque::new(),
                read_cb: None,
            },
        );

        Some(fd)
    }

    /// `host` as the four address bytes in dial order, `port` in host order. False when nothing
    /// listed answers there.
    pub fn connect(&mut self, fd: i32, host: [u8; 4], port: u16) -> bool {
        let Some(server) = SERVERS.iter().find(|x| x.aid == self.aid && x.host == host && x.port == port) else {
            tracing::warn!(
                "fake server: nothing at {}.{}.{}.{}:{port} for {}",
                host[0],
                host[1],
                host[2],
                host[3],
                self.aid
            );
            return false;
        };
        match self.sockets.get_mut(&fd) {
            Some(socket) => {
                socket.respond = Some(server.respond);
                true
            }
            None => false,
        }
    }

    /// Takes all of `data`; the reply, if a request is now complete, is queued for `read`. Returns
    /// whether a reply was queued, None for a socket that is not connected.
    pub fn write(&mut self, fd: i32, data: &[u8]) -> Option<bool> {
        let socket = self.sockets.get_mut(&fd)?;
        let respond = socket.respond?;
        socket.request.extend_from_slice(data);
        let Some(reply) = respond(&mut socket.request) else {
            return Some(false);
        };
        tracing::debug!("fake server: {} replies {} bytes", self.aid, reply.len());
        socket.reply.extend(reply);

        Some(true)
    }

    /// Up to `len` reply bytes; empty when none are queued (the caller's «would block»). None for a
    /// socket that is not connected.
    pub fn read(&mut self, fd: i32, len: usize) -> Option<Vec<u8>> {
        let socket = self.sockets.get_mut(&fd)?;
        socket.respond?;
        let n = len.min(socket.reply.len());

        Some(socket.reply.drain(..n).collect())
    }

    pub fn readable(&self, fd: i32) -> bool {
        self.sockets.get(&fd).is_some_and(|x| !x.reply.is_empty())
    }

    pub fn set_read_cb(&mut self, fd: i32, cb: u32, param: u32) {
        if let Some(socket) = self.sockets.get_mut(&fd) {
            socket.read_cb = Some((cb, param));
        }
    }

    pub fn take_read_cb(&mut self, fd: i32) -> Option<(u32, u32)> {
        self.sockets.get_mut(&fd)?.read_cb.take()
    }

    pub fn close(&mut self, fd: i32) -> bool {
        self.sockets.remove(&fd).is_some()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::FakeNet;

    // A title with no listed server keeps the old answers: no socket, so every caller's
    // «no network» branch is untouched.
    #[test]
    fn unlisted_title_stays_offline() {
        let mut net = FakeNet::new("00000000");
        assert!(!net.online());
        assert_eq!(net.socket(), None);
    }

    // A listed title still gets nothing from a host that is not its own server — another listed
    // title's included.
    #[test]
    fn listed_title_reaches_only_its_own_host() {
        let mut net = FakeNet::new("00026E9C");
        let fd = net.socket().unwrap();
        assert!(!net.connect(fd, [211, 115, 66, 250], 15133));
        assert_eq!(net.write(fd, &[4, 0, 2, 1]), None);
        assert_eq!(net.read(fd, 4), None);
    }

    // caf9d76ffd13's login, split across two writes the way a short write would leave it: no reply
    // until the whole packet is in, then `0x0103` with result 1 and the «done» byte.
    #[test]
    fn login_0102_is_answered_once_complete() {
        let mut net = FakeNet::new("00026E9C");
        let fd = net.socket().unwrap();
        assert!(net.connect(fd, [218, 145, 70, 36], 30806));
        let mut login = vec![0x3f, 0, 0x02, 0x01];
        login.resize(0x3f, 0);
        assert_eq!(net.write(fd, &login[..10]), Some(false));
        assert_eq!(net.read(fd, 4), Some(vec![]));
        assert_eq!(net.write(fd, &login[10..]), Some(true));

        let reply = net.read(fd, 0x100).unwrap();
        assert_eq!(reply.len(), 46);
        assert_eq!(&reply[..5], &[46, 0, 0x03, 0x01, 1]);
        assert_eq!(reply[45], 1);
        // a command it does not know gets no answer rather than a guess
        assert_eq!(net.write(fd, &[4, 0, 0x00, 0x02]), Some(false));
    }

    // b9bfcaf42722: hello, then the certificate with its length at offset 6.
    #[test]
    fn certificate_c2s_answers_hello_then_certificate() {
        let mut net = FakeNet::new("0002A61E");
        let fd = net.socket().unwrap();
        assert!(net.connect(fd, [211, 115, 66, 250], 15133));
        let mut hello = vec![0, 0x49, 0, 0x00];
        hello.resize(0x49, 0);
        assert_eq!(net.write(fd, &hello), Some(true));
        assert_eq!(net.read(fd, 2), Some(vec![0, 6]));
        assert_eq!(net.read(fd, 4), Some(vec![0, 0x00, 0, 0]));

        let mut ask = vec![0, 0x4f, 0, 0x14];
        ask.resize(0x4f, 0);
        assert_eq!(net.write(fd, &ask), Some(true));
        let reply = net.read(fd, 0x100).unwrap();
        assert_eq!(&reply[..8], &[0, 24, 0, 0x14, 0, 0, 0, 16]);
        assert_eq!(reply.len(), 24);
        assert!(net.close(fd));
    }
}
