import { ALL_KEYS, type EmuKey, loadKeymap } from "../lib/keymap";

// Human-friendly label for a KeyboardEvent.code.
function codeLabel(code: string): string {
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code.startsWith("Arrow")) return { ArrowUp: "↑", ArrowDown: "↓", ArrowLeft: "←", ArrowRight: "→" }[code] ?? code;
  return { Space: "Space", Enter: "Enter", Backspace: "Backspace", ShiftLeft: "Shift(L)", ShiftRight: "Shift(R)", Tab: "Tab", Backquote: "`" }[code] ?? code;
}

const KEY_LABELS: Record<EmuKey, string> = {
  UP: "위", DOWN: "아래", LEFT: "왼쪽", RIGHT: "오른쪽", OK: "확인(OK)",
  LEFT_SOFT_KEY: "왼쪽 소프트키", RIGHT_SOFT_KEY: "오른쪽 소프트키", CLEAR: "지우기(CLR)",
  CALL: "통화", HANGUP: "종료", VOLUME_UP: "볼륨 +", VOLUME_DOWN: "볼륨 -",
  NUM0: "0", NUM1: "1", NUM2: "2", NUM3: "3", NUM4: "4", NUM5: "5", NUM6: "6", NUM7: "7", NUM8: "8", NUM9: "9",
  STAR: "*", HASH: "#",
};

// The key table is read from the CURRENT keymap (localStorage) so it always
// matches what the user actually has bound — change the mapping and the help
// updates with it. Nothing here is sent to the server.
export function Help() {
  const map = loadKeymap();
  const bindingsFor = (k: EmuKey) =>
    Object.entries(map)
      .filter(([, v]) => v === k)
      .map(([code]) => codeLabel(code));

  return (
    <div className="w-full max-w-xl text-sm text-fg-dim flex flex-col gap-4">
      <section>
        <h3 className="font-semibold text-fg mb-1">WIE는 무엇인가요?</h3>
        <p>
          WIPI·SKVM·J2ME 기반 옛 피처폰 게임을 <strong>브라우저에서 그대로 실행</strong>하는 에뮬레이터입니다. 설치가 필요 없고, 게임 파일은
          전부 <strong>당신의 브라우저 안에서만</strong> 처리됩니다. 업로드한 게임 파일은 서버로 전송되지 않으며(BYOF), 서버에는 계정 정보와
          세이브 데이터만 저장됩니다.
        </p>
      </section>

      <section>
        <h3 className="font-semibold text-fg mb-1">게임 파일 올리는 법</h3>
        <ul className="list-disc list-inside space-y-1">
          <li><strong>.jar</strong> — WIPI(KTF/LGT)·SKVM·J2ME(MIDP) 게임 JAR 파일 한 개.</li>
          <li><strong>.jad + .jar</strong> — J2ME 디스크립터와 JAR을 함께 선택(JAR가 실제 실행 파일).</li>
          <li><strong>.zip</strong> — KTF/LGT/SKT 패키지(보통 안에 JAR 등이 들어 있는 압축).</li>
        </ul>
        <p className="mt-1 text-xs">라이브러리에 추가하면 이 기기(IndexedDB)에 보관되어, 다음에 재업로드 없이 바로 실행됩니다.</p>
      </section>

      <section>
        <h3 className="font-semibold text-fg mb-1">키 조작 (현재 내 설정)</h3>
        <p className="text-xs mb-2">아래 표는 현재 활성화된 키 매핑을 그대로 보여줍니다. 플레이어의 “키 설정”에서 바꾸면 이 표도 함께 바뀝니다.</p>
        <div className="grid grid-cols-2 gap-x-4 gap-y-0.5">
          {ALL_KEYS.map((k) => {
            const keys = bindingsFor(k);
            return (
              <div key={k} className="flex items-center justify-between gap-2 py-0.5 border-b border-edge/50">
                <span>{KEY_LABELS[k]}</span>
                <span className={keys.length ? "text-fg font-medium" : "text-amber-600 dark:text-amber-300"}>
                  {keys.join(", ") || "미설정"}
                </span>
              </div>
            );
          })}
        </div>
        <p className="mt-2 text-xs">화면의 가상 패드(터치/클릭)로도 조작할 수 있습니다.</p>
      </section>
    </div>
  );
}
