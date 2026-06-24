import { useCallback, useEffect, useRef, useState } from "react";
import { inquiries, type Inquiry, type InquiryAttachment, type User } from "../lib/api";
import { envInfoText } from "../lib/device";

interface Props {
  user: User | null;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

const MAX_ATTACH = 96 * 1024;
const ALLOWED_MIME = /^(image\/(png|jpeg|gif|webp|bmp)|text\/plain)$/;
const BLOCKED_EXT = /\.(jar|jad|zip|kdf|skm|mod|smc|gba|nes|class|exe|dll|so|bin|apk|7z|rar|gz)$/i;
// Magic numbers of game/exec/archive files — blocked even if mislabeled.
const BLOCKED_MAGICS: number[][] = [
  [0x50, 0x4b, 0x03, 0x04],
  [0xca, 0xfe, 0xba, 0xbe],
  [0x7f, 0x45, 0x4c, 0x46],
  [0x4d, 0x5a],
  [0x1f, 0x8b],
  [0x52, 0x61, 0x72, 0x21],
  [0x37, 0x7a, 0xbc, 0xaf],
];

function bytesToB64(bytes: Uint8Array): string {
  let bin = "";
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin);
}

// Validate + read an attachment client-side (mirrors the server's 415 policy so
// the user gets instant feedback). Throws Error with a Korean message on reject.
async function readAttachment(file: File): Promise<InquiryAttachment> {
  const buf = new Uint8Array(await file.arrayBuffer());
  if (buf.length > MAX_ATTACH) throw new Error("첨부 파일이 너무 큽니다 (최대 96KB)");
  if (BLOCKED_EXT.test(file.name)) throw new Error("게임/실행 파일은 첨부할 수 없습니다");
  if (!ALLOWED_MIME.test(file.type || "")) throw new Error("이미지·텍스트·로그 파일만 첨부할 수 있습니다");
  if (BLOCKED_MAGICS.some((m) => m.every((b, i) => buf[i] === b))) throw new Error("게임/실행 파일은 첨부할 수 없습니다");
  return { name: file.name, mime: file.type, data: bytesToB64(buf) };
}

export function InquiryForm({ user, toast }: Props) {
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [attach, setAttach] = useState<InquiryAttachment | null>(null);
  const [busy, setBusy] = useState(false);
  const [showEnv, setShowEnv] = useState(false);
  const [list, setList] = useState<Inquiry[]>([]);
  const fileRef = useRef<HTMLInputElement>(null);
  const env = envInfoText();

  const refresh = useCallback(async () => {
    if (!user) return;
    try {
      setList((await inquiries.list()).inquiries);
    } catch {
      /* ignore */
    }
  }, [user]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const onPickFile = async (f: File | undefined) => {
    if (!f) return setAttach(null);
    try {
      setAttach(await readAttachment(f));
    } catch (e) {
      setAttach(null);
      if (fileRef.current) fileRef.current.value = "";
      toast((e as Error).message, "err");
    }
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setBusy(true);
    try {
      await inquiries.create({ title: title.trim(), body: body.trim(), env_info: env, attachment: attach });
      toast("문의가 접수되었습니다 (환경정보 자동 첨부)", "ok");
      setTitle("");
      setBody("");
      setAttach(null);
      if (fileRef.current) fileRef.current.value = "";
      await refresh();
    } catch (err) {
      toast(`전송 실패: ${(err as Error).message}`, "err");
    } finally {
      setBusy(false);
    }
  };

  if (!user) {
    return (
      <section className="w-full max-w-xl">
        <h2 className="mb-2 text-lg font-semibold text-fg">문의 · 건의</h2>
        <div className="rounded-lg border border-dashed border-edge bg-surface2 p-4 text-fg-dim">문의는 로그인 회원만 작성할 수 있습니다.</div>
      </section>
    );
  }

  const input = "rounded-md bg-surface2 border border-edge px-3 py-2 text-fg text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";

  return (
    <section className="w-full max-w-xl flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">문의 · 건의</h2>
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm text-fg-dim">
          제목
          <input value={title} onChange={(e) => setTitle(e.target.value)} maxLength={200} required className={input} />
        </label>
        <label className="flex flex-col gap-1 text-sm text-fg-dim">
          내용
          <textarea value={body} onChange={(e) => setBody(e.target.value)} rows={6} maxLength={8000} required className={input} />
        </label>

        <div className="flex flex-col gap-1 text-sm text-fg-dim">
          파일 첨부 (선택 · 이미지/텍스트/로그만)
          <input
            ref={fileRef}
            type="file"
            accept="image/png,image/jpeg,image/gif,image/webp,image/bmp,text/plain,.txt,.log"
            onChange={(e) => void onPickFile(e.target.files?.[0])}
            className="text-xs text-fg-dim file:mr-2 file:rounded file:border file:border-edge file:bg-surface2 file:px-2 file:py-1 file:text-fg"
          />
          {attach && <span className="text-xs text-emerald-600 dark:text-emerald-300">첨부됨: {attach.name} ({attach.mime})</span>}
          <span className="text-[11px] text-fg-dim">게임/실행 파일(JAR·ROM·ZIP 등)은 첨부할 수 없으며 서버가 거부합니다(415). 최대 96KB.</span>
        </div>

        <div className="rounded-md border border-edge bg-surface2 px-3 py-2 text-xs text-fg-dim">
          <button type="button" onClick={() => setShowEnv((v) => !v)} className="underline">
            {showEnv ? "자동 첨부 환경정보 숨기기" : "자동으로 첨부될 환경정보 보기"}
          </button>
          {showEnv && <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap break-words text-[11px] text-fg">{env}</pre>}
          <p className="mt-1">접수 시 위 환경정보가 자동 첨부됩니다. 게임 파일명·제목 등 게임 식별정보는 포함되지 않습니다.</p>
        </div>

        <button type="submit" disabled={busy} className="self-start rounded-md bg-accent hover:opacity-90 disabled:opacity-60 px-4 py-2 font-medium text-accent-fg">
          {busy ? "전송 중…" : "보내기"}
        </button>
      </form>

      <h3 className="text-sm font-semibold text-fg-dim">내 문의 내역</h3>
      <ul className="flex flex-col gap-2">
        {list.length === 0 && <li className="text-sm text-fg-dim">내역이 없습니다.</li>}
        {list.map((q) => (
          <li key={q.id} className="rounded-lg border border-edge bg-surface2 px-3 py-2">
            <div className="font-medium text-fg">
              {q.title}
              {q.has_attachment ? <span className="ml-2 rounded bg-surface px-1.5 py-0.5 text-[10px] text-fg-dim">📎 첨부</span> : null}
            </div>
            <div className="text-xs text-fg-dim">
              {new Date(q.created_at).toLocaleString()} · {q.status}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
