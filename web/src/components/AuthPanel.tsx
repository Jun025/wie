import { useState } from "react";
import type { AuthState } from "../hooks/useAuth";
import { deviceName, setDeviceName } from "../lib/saveSync";

interface Props {
  authState: AuthState;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

export function AuthPanel({ authState, toast }: Props) {
  const { user, login, register, logout } = authState;
  const [mode, setMode] = useState<"login" | "register">("login");
  const [loginId, setLoginId] = useState("");
  const [password, setPassword] = useState("");
  const [email, setEmail] = useState("");
  const [error, setError] = useState("");
  const [devName, setDevName] = useState(deviceName());

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
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
      setError((err as Error).message);
    }
  };

  if (user) {
    return (
      <section className="w-full max-w-md flex flex-col gap-4">
        <h2 className="text-lg font-semibold text-slate-100">계정</h2>
        <p className="text-slate-300">
          로그인됨: <strong>@{user.login_id}</strong>
        </p>
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          이 기기 별칭
          <input
            value={devName}
            maxLength={60}
            onChange={(e) => setDevName(e.target.value)}
            onBlur={() => {
              setDeviceName(devName.trim());
              toast("기기 별칭 저장됨");
            }}
            className="rounded-md bg-slate-800 border border-slate-700 px-3 py-2 text-slate-100"
          />
        </label>
        <button
          type="button"
          onClick={() => void logout().then(() => toast("로그아웃됨"))}
          className="self-start rounded-md bg-slate-700 hover:bg-slate-600 px-4 py-2 text-sm text-slate-200"
        >
          로그아웃
        </button>
      </section>
    );
  }

  return (
    <section className="w-full max-w-md flex flex-col gap-4">
      <h2 className="text-lg font-semibold text-slate-100">계정</h2>
      <div className="flex gap-2">
        {(["login", "register"] as const).map((m) => (
          <button
            key={m}
            type="button"
            onClick={() => setMode(m)}
            className={"px-3 py-1 rounded text-sm " + (mode === m ? "bg-slate-700 text-white" : "text-slate-400 hover:text-slate-200")}
          >
            {m === "login" ? "로그인" : "가입"}
          </button>
        ))}
      </div>
      <form onSubmit={submit} className="flex flex-col gap-3">
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          아이디 / 이메일
          <input
            value={loginId}
            onChange={(e) => setLoginId(e.target.value)}
            autoComplete="username"
            required
            minLength={3}
            className="rounded-md bg-slate-800 border border-slate-700 px-3 py-2 text-slate-100"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm text-slate-400">
          비밀번호
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            autoComplete={mode === "register" ? "new-password" : "current-password"}
            required
            minLength={8}
            className="rounded-md bg-slate-800 border border-slate-700 px-3 py-2 text-slate-100"
          />
        </label>
        {mode === "register" && (
          <label className="flex flex-col gap-1 text-sm text-slate-400">
            이메일 (선택)
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              autoComplete="email"
              className="rounded-md bg-slate-800 border border-slate-700 px-3 py-2 text-slate-100"
            />
          </label>
        )}
        {error && <p className="text-sm text-red-400">{error}</p>}
        <button type="submit" className="rounded-md bg-sky-600 hover:bg-sky-500 px-4 py-2 font-medium text-white">
          {mode === "register" ? "가입하기" : "로그인"}
        </button>
      </form>
      <p className="text-xs text-slate-500">비밀번호는 PBKDF2-HMAC-SHA256으로 해시되어 저장되며 평문은 서버에 남지 않습니다.</p>
    </section>
  );
}
