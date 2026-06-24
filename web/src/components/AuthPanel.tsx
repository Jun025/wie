import { useState } from "react";
import type { AuthState } from "../hooks/useAuth";
import { deviceName, setDeviceName } from "../lib/saveSync";

interface Props {
  authState: AuthState;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

// Shared theme-token classes so the panel matches light/dark like the rest of
// the app (it previously used hardcoded slate colours that looked broken in
// light mode).
const inputCls = "rounded-md bg-surface2 border border-edge px-3 py-2 text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
const labelCls = "flex flex-col gap-1 text-sm text-fg-dim";

export function AuthPanel({ authState, toast }: Props) {
  const { user, login, register, logout } = authState;
  const [mode, setMode] = useState<"login" | "register">("login");
  const [loginId, setLoginId] = useState("");
  const [password, setPassword] = useState("");
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [devName, setDevName] = useState(deviceName());

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setBusy(true);
    try {
      if (mode === "register") {
        await register(loginId.trim(), password, email || undefined);
        toast("가입 완료", "ok");
      } else {
        await login(loginId.trim(), password);
        toast("로그인 완료", "ok");
      }
      setPassword("");
    } catch (err) {
      // Surface backend errors clearly (중복 ID / 잘못된 비번 / rate-limit 429 / 네트워크).
      setError((err as Error).message || "요청을 처리하지 못했습니다");
    } finally {
      setBusy(false);
    }
  };

  if (user) {
    return (
      <section className="w-full max-w-md flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-fg">계정</h2>
        <p className="text-fg">
          로그인됨: <strong>@{user.login_id}</strong>
        </p>
        <label className={labelCls}>
          이 기기 별칭
          <input
            value={devName}
            maxLength={60}
            onChange={(e) => setDevName(e.target.value)}
            onBlur={() => {
              setDeviceName(devName.trim());
              toast("기기 별칭 저장됨");
            }}
            className={inputCls}
          />
        </label>
        <button
          type="button"
          onClick={() => void logout().then(() => toast("로그아웃됨"))}
          className="self-start rounded-md border border-edge bg-surface2 hover:border-accent px-4 py-2 text-sm text-fg"
        >
          로그아웃
        </button>
      </section>
    );
  }

  return (
    <section className="w-full max-w-md flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">계정</h2>
      <div className="flex gap-2">
        {(["login", "register"] as const).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => {
              setMode(m);
              setError("");
            }}
            aria-pressed={mode === m}
            className={"px-3 py-1 rounded text-sm " + (mode === m ? "bg-accent text-accent-fg" : "text-fg-dim hover:text-fg")}
          >
            {m === "login" ? "로그인" : "가입"}
          </button>
        ))}
      </div>
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className={labelCls}>
          아이디 / 이메일
          <input value={loginId} onChange={(e) => setLoginId(e.target.value)} autoComplete="username" required minLength={3} className={inputCls} />
        </label>
        <label className={labelCls}>
          비밀번호
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete={mode === "register" ? "new-password" : "current-password"}
            required
            minLength={8}
            className={inputCls}
          />
        </label>
        {mode === "register" && (
          <label className={labelCls}>
            이메일 (선택)
            <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" className={inputCls} />
          </label>
        )}
        {error && (
          <p role="alert" className="text-sm text-red-600 dark:text-red-300">
            {error}
          </p>
        )}
        <button
          type="submit"
          disabled={busy}
          className="rounded-md bg-accent hover:opacity-90 disabled:opacity-60 px-4 py-2 font-medium text-accent-fg"
        >
          {busy ? "처리 중…" : mode === "register" ? "가입하기" : "로그인"}
        </button>
      </form>
      <p className="text-xs text-fg-dim">비밀번호는 PBKDF2-HMAC-SHA256으로 해시되어 저장되며 평문은 서버에 남지 않습니다.</p>
    </section>
  );
}
