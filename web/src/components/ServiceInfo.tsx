import { clientEnv, APP_VERSION } from "../lib/device";
import { ReportForm } from "./ReportForm";

// "서비스 정보" — distribution/license notices (moved here from the main screen)
// plus a read-only view of THIS device/network (client-side only, never sent to
// the server, no over-collection). The MIT license + original author + upstream
// link are preserved here (S7).
export function ServiceInfo({ toast }: { toast: (msg: string, kind?: "ok" | "err") => void }) {
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
        <h3 className="text-sm font-semibold text-fg">이용 약관 · 고지</h3>
        <ul className="mt-2 space-y-1.5 text-fg-dim">
          <li>• 이용자는 <span className="text-fg">본인이 보유한 적법한 파일</span>만 업로드할 수 있습니다. 권리 없는 파일의 업로드는 금지됩니다.</li>
          <li>• 업로드한 게임 파일은 <span className="text-fg">개인 보관 목적</span>의 1인 전용 보관함에 저장되며, <span className="text-fg">공유·공개·배포·재전송이 금지</span>됩니다. 다른 이용자는 회원님의 파일에 접근할 수 없습니다.</li>
          <li>• 미로그인 상태에서 추가한 게임 파일은 <span className="text-fg">이 기기(브라우저)에만</span> 저장되어 캐시 삭제·기기 변경 시 <span className="text-fg">소실될 수 있습니다</span>(기기 종속). 중요한 데이터는 직접 백업하세요.</li>
          <li>• 본 서비스는 파일 단위 과금이나 광고 연동을 하지 않으며, 게임 파일을 판매·배포·중개하지 않습니다.</li>
          <li>• 권리자의 신고가 접수되면 대상 파일을 즉시 비활성화하고, 반복 침해 계정은 이용을 제한·해지합니다.</li>
        </ul>
      </div>

      <div className="rounded-lg border border-edge bg-surface2 p-4 text-sm">
        <h3 className="text-sm font-semibold text-fg">권리 침해 신고 · 삭제 요청</h3>
        <p className="mb-2 mt-1 text-xs text-fg-dim">권리자(또는 대리인)는 아래로 침해 사실을 신고할 수 있습니다. 검토 후 대상 파일을 비활성화하는 등 신속히 조치합니다.</p>
        <ReportForm toast={toast} />
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
          <li>에뮬레이터 코어 자체는 게임 파일을 포함·배포하지 않습니다 — 이용자가 직접 가져온 파일만 처리합니다(BYOF).</li>
        </ul>
      </div>
    </section>
  );
}
