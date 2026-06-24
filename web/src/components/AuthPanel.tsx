import { useState } from "react";
import type { AuthState } from "../hooks/useAuth";
import { auth, ApiError } from "../lib/api";
import { DeviceStatus } from "./DeviceStatus";

interface Props {
  authState: AuthState;
  toast: (msg: string, kind?: "ok" | "err") => void;
  resetToken?: string | null; // present → show "set new password" flow
  onResetDone?: () => void; // clear the ?reset= token from the URL/state
}

const inputCls = "rounded-md bg-surface2 border border-edge px-3 py-2 text-fg focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent";
const labelCls = "flex flex-col gap-1 text-sm text-fg-dim";

// Password field with a show/hide toggle.
function PasswordField({ value, onChange, autoComplete, placeholder, id }: { value: string; onChange: (v: string) => void; autoComplete: string; placeholder?: string; id?: string }) {
  const [show, setShow] = useState(false);
  return (
    <div className="relative">
      <input
        id={id}
        type={show ? "text" : "password"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        autoComplete={autoComplete}
        placeholder={placeholder}
        required
        minLength={8}
        className={`${inputCls} w-full pr-12`}
      />
      <button
        type="button"
        onClick={() => setShow((s) => !s)}
        aria-label={show ? "비밀번호 숨기기" : "비밀번호 표시"}
        className="absolute inset-y-0 right-0 px-3 text-xs text-fg-dim hover:text-fg"
      >
        {show ? "숨김" : "표시"}
      </button>
    </div>
  );
}

export function AuthPanel({ authState, toast, resetToken, onResetDone }: Props) {
  const { user, login, register, logout, emailConfigured } = authState;
  const [mode, setMode] = useState<"login" | "register" | "forgot">("login");
  const [loginId, setLoginId] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [pendingNotice, setPendingNotice] = useState<string | null>(null); // after register-with-verify
  const [unverifiedId, setUnverifiedId] = useState<string | null>(null); // login blocked: offer resend

  // ── Password-reset completion (arrived via the emailed ?reset=TOKEN link) ─────
  if (resetToken) {
    return <ResetForm token={resetToken} toast={toast} onDone={onResetDone} />;
  }

  const resend = async (id: string) => {
    try {
      await auth.resend(id);
      toast("인증 메일을 다시 보냈습니다 (등록된 주소가 있는 경우)", "ok");
    } catch (e) {
      toast(`재발송 실패: ${(e as Error).message}`, "err");
    }
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setUnverifiedId(null);
    setPendingNotice(null);

    if (mode === "forgot") {
      setBusy(true);
      try {
        const r = await auth.requestReset(loginId.trim());
        toast(r.emailConfigured ? "재설정 메일을 보냈습니다 (등록된 주소가 있는 경우 도착합니다)" : "이메일 기능이 설정되지 않아 메일을 보낼 수 없습니다", r.emailConfigured ? "ok" : "err");
      } catch (err) {
        setError((err as Error).message);
      } finally {
        setBusy(false);
      }
      return;
    }

    if (mode === "register") {
      if (password !== confirm) {
        setError("비밀번호가 일치하지 않습니다");
        return;
      }
      if (emailConfigured && !email.trim()) {
        setError("이메일 인증을 위해 이메일을 입력해 주세요");
        return;
      }
    }

    setBusy(true);
    try {
      if (mode === "register") {
        const res = await register(loginId.trim(), password, email.trim() || undefined);
        if (res.pending) {
          setPendingNotice("인증 메일을 보냈습니다. 메일함(스팸함 포함)에서 링크를 열어 인증을 완료한 뒤 로그인하세요. 메일이 안 오면 아래에서 다시 보낼 수 있습니다.");
          setUnverifiedId(loginId.trim());
          setMode("login");
        } else if (emailConfigured && email.trim() && !res.emailSent) {
          // Email configured but the verification mail couldn't be delivered
          // (e.g. Resend test mode → only the account owner receives mail). We
          // activated the account instead of locking it out.
          toast("가입 완료 — 인증 메일을 보내지 못해 바로 로그인했습니다(서비스 설정에 따라 일부 주소는 수신이 제한될 수 있어요).", "ok");
        } else {
          toast("가입 완료", "ok");
        }
      } else {
        await login(loginId.trim(), password);
        toast("로그인 완료", "ok");
      }
      setPassword("");
      setConfirm("");
    } catch (err) {
      if (err instanceof ApiError && err.code === "email_unverified") {
        setUnverifiedId(loginId.trim());
        setError(err.message);
      } else {
        setError((err as Error).message || "요청을 처리하지 못했습니다");
      }
    } finally {
      setBusy(false);
    }
  };

  // ── Logged in ────────────────────────────────────────────────────────────────
  if (user) {
    const verified = !!user.email_verified;
    return (
      <section className="w-full max-w-md flex flex-col gap-5">
        <div>
          <h2 className="text-lg font-semibold text-fg">계정</h2>
          <p className="mt-1 text-fg">
            로그인됨: <strong>@{user.login_id}</strong>
          </p>
          {user.email && (
            <p className="mt-1 text-sm text-fg-dim">
              {user.email}{" "}
              {verified ? (
                <span className="rounded bg-emerald-500/20 px-1.5 py-0.5 text-[11px] text-emerald-600 dark:text-emerald-300">인증됨</span>
              ) : (
                <span className="rounded bg-amber-500/20 px-1.5 py-0.5 text-[11px] text-amber-600 dark:text-amber-300">미인증</span>
              )}
            </p>
          )}
          {user.email && !verified && emailConfigured && (
            <button type="button" onClick={() => void resend(user.login_id)} className="mt-2 rounded-md border border-edge bg-surface2 px-3 py-1.5 text-xs text-fg hover:border-accent">
              인증 메일 다시 보내기
            </button>
          )}
        </div>

        <DeviceStatus toast={toast} />

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

  // ── Logged out: login / register / forgot ────────────────────────────────────
  return (
    <section className="w-full max-w-md flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">계정</h2>

      {pendingNotice && (
        <div className="rounded-md border border-amber-500/50 bg-amber-500/10 px-3 py-2 text-sm text-amber-700 dark:text-amber-200">{pendingNotice}</div>
      )}

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
            className={"px-3 py-1 rounded text-sm " + (mode === m || (m === "login" && mode === "forgot") ? "bg-accent text-accent-fg" : "text-fg-dim hover:text-fg")}
          >
            {m === "login" ? "로그인" : "가입"}
          </button>
        ))}
      </div>

      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className={labelCls}>
          아이디{mode === "register" ? " (3자 이상)" : ""}
          <input value={loginId} onChange={(e) => setLoginId(e.target.value)} autoComplete="username" required minLength={mode === "register" ? 3 : 1} className={inputCls} />
        </label>

        {mode !== "forgot" && (
          <label className={labelCls}>
            비밀번호{mode === "register" ? " (8자 이상)" : ""}
            <PasswordField value={password} onChange={setPassword} autoComplete={mode === "register" ? "new-password" : "current-password"} />
          </label>
        )}

        {mode === "register" && (
          <>
            <label className={labelCls}>
              비밀번호 확인
              <PasswordField value={confirm} onChange={setConfirm} autoComplete="new-password" />
              {confirm && password !== confirm && <span className="text-xs text-red-600 dark:text-red-300">비밀번호가 일치하지 않습니다</span>}
            </label>
            <label className={labelCls}>
              이메일{emailConfigured ? " (인증에 사용)" : " (선택)"}
              <input type="email" value={email} onChange={(e) => setEmail(e.target.value)} autoComplete="email" required={emailConfigured} className={inputCls} />
            </label>
            {emailConfigured ? (
              <p className="text-xs text-fg-dim">가입 후 인증 메일의 링크를 열면 로그인할 수 있습니다. 메일이 안 오면 스팸함을 확인하거나 재발송하세요(서비스 설정에 따라 일부 주소는 수신이 제한될 수 있습니다).</p>
            ) : (
              <p className="text-xs text-fg-dim">현재 이메일 인증 기능이 설정되어 있지 않아, 가입 즉시 사용할 수 있습니다.</p>
            )}
          </>
        )}

        {mode === "forgot" && <p className="text-xs text-fg-dim">가입 시 등록한 이메일로 재설정 링크를 보냅니다. (이메일 기능이 설정된 경우)</p>}

        {error && (
          <p role="alert" className="text-sm text-red-600 dark:text-red-300">
            {error}
          </p>
        )}
        {unverifiedId && (
          <button type="button" onClick={() => void resend(unverifiedId)} className="self-start rounded-md border border-edge bg-surface2 px-3 py-1.5 text-xs text-fg hover:border-accent">
            인증 메일 다시 보내기
          </button>
        )}

        <button type="submit" disabled={busy} className="rounded-md bg-accent hover:opacity-90 disabled:opacity-60 px-4 py-2 font-medium text-accent-fg">
          {busy ? "처리 중…" : mode === "register" ? "가입하기" : mode === "forgot" ? "재설정 메일 보내기" : "로그인"}
        </button>
      </form>

      {mode !== "forgot" ? (
        <button type="button" onClick={() => { setMode("forgot"); setError(""); }} className="self-start text-xs text-fg-dim underline hover:text-fg">
          비밀번호를 잊으셨나요?
        </button>
      ) : (
        <button type="button" onClick={() => { setMode("login"); setError(""); }} className="self-start text-xs text-fg-dim underline hover:text-fg">
          로그인으로 돌아가기
        </button>
      )}

      <p className="text-xs text-fg-dim">비밀번호는 PBKDF2-HMAC-SHA256으로 해시되어 저장되며 평문은 서버에 남지 않습니다.</p>
    </section>
  );
}

// Set-new-password form shown when arriving from the emailed reset link.
function ResetForm({ token, toast, onDone }: { token: string; toast: (m: string, k?: "ok" | "err") => void; onDone?: () => void }) {
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    if (password !== confirm) return setError("비밀번호가 일치하지 않습니다");
    setBusy(true);
    try {
      await auth.reset(token, password);
      setDone(true);
      toast("비밀번호가 변경되었습니다. 새 비밀번호로 로그인하세요.", "ok");
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setBusy(false);
    }
  };

  if (done) {
    return (
      <section className="w-full max-w-md flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-fg">비밀번호 재설정 완료</h2>
        <p className="text-sm text-fg-dim">이제 새 비밀번호로 로그인할 수 있습니다.</p>
        <button type="button" onClick={onDone} className="self-start rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-fg hover:opacity-90">
          로그인 화면으로
        </button>
      </section>
    );
  }

  return (
    <section className="w-full max-w-md flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-fg">새 비밀번호 설정</h2>
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className={labelCls}>
          새 비밀번호 (8자 이상)
          <PasswordField value={password} onChange={setPassword} autoComplete="new-password" />
        </label>
        <label className={labelCls}>
          새 비밀번호 확인
          <PasswordField value={confirm} onChange={setConfirm} autoComplete="new-password" />
          {confirm && password !== confirm && <span className="text-xs text-red-600 dark:text-red-300">비밀번호가 일치하지 않습니다</span>}
        </label>
        {error && <p role="alert" className="text-sm text-red-600 dark:text-red-300">{error}</p>}
        <button type="submit" disabled={busy} className="rounded-md bg-accent hover:opacity-90 disabled:opacity-60 px-4 py-2 font-medium text-accent-fg">
          {busy ? "처리 중…" : "비밀번호 변경"}
        </button>
      </form>
    </section>
  );
}
