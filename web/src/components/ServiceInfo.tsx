import { clientEnv, APP_VERSION } from "../lib/device";

// "서비스 정보" — distribution/license notices (moved here from the main screen)
// plus a read-only view of THIS device/network (client-side only, never sent to
// the server, no over-collection). The MIT license + original author + upstream
// link are preserved here (S7).
export function ServiceInfo() {
  const e = clientEnv();
  const rows: [string, string][] = [
    ["브라우저", e.browser],
    ["운영체제", e.os],
    ["화면", e.screen],
    ["CPU 코어", e.cores],
    ["네트워크", e.network],
    ["언어", e.language],
    ["앱 버전", APP_VERSION],
  ];

  return (
    <section className="w-full max-w-xl flex flex-col gap-5">
      <div>
        <h2 className="text-lg font-semibold text-fg">서비스 정보</h2>
        <p className="mt-1 text-sm text-fg-dim">
          WIPI·SKVM·J2ME 기반 옛 피처폰 게임을 브라우저에서 실행하는 에뮬레이터입니다. 게임 파일은 이 기기(브라우저)에서만 처리되며 서버로 전송되지 않습니다.
        </p>
      </div>

      <div className="rounded-lg border border-edge bg-surface2 p-4">
        <h3 className="text-sm font-semibold text-fg">이 기기 · 네트워크 정보</h3>
        <p className="mb-2 mt-1 text-xs text-fg-dim">아래 정보는 이 브라우저에서 읽어 화면에만 표시됩니다(서버 전송 없음).</p>
        <dl className="grid grid-cols-[7rem_1fr] gap-x-3 gap-y-1 text-sm">
          {rows.map(([k, v]) => (
            <div key={k} className="contents">
              <dt className="text-fg-dim">{k}</dt>
              <dd className="break-words text-fg">{v}</dd>
            </div>
          ))}
          <div className="contents">
            <dt className="text-fg-dim">User-Agent</dt>
            <dd className="break-all text-xs text-fg-dim">{e.userAgent}</dd>
          </div>
        </dl>
      </div>

      <div className="rounded-lg border border-edge bg-surface2 p-4 text-sm">
        <h3 className="text-sm font-semibold text-fg">저작권 · 라이선스 · 배포</h3>
        <ul className="mt-2 space-y-1 text-fg-dim">
          <li>
            에뮬레이터 코어: <span className="text-fg">MIT 라이선스</span> · © 2020 Inseok Lee
          </li>
          <li>
            업스트림:{" "}
            <a className="text-accent underline hover:opacity-80" href="https://github.com/dlunch/wie" target="_blank" rel="noreferrer noopener">
              github.com/dlunch/wie
            </a>
          </li>
          <li>이 웹 서비스는 위 오픈소스 코어를 브라우저에서 실행하기 위한 프런트엔드/백엔드 레이어입니다.</li>
          <li>게임 파일(JAR/ROM 등)은 보관·배포하지 않습니다 — 이용자가 직접 가져오는 파일은 브라우저에만 보관됩니다(BYOF).</li>
        </ul>
      </div>
    </section>
  );
}
