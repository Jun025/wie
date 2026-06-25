-- wie web service — make EMAIL the sole login identifier (remove username/login_id).
--
-- ★사용자 승인: 서비스가 아직 미운영이므로 기존 유저/서비스 데이터를 전부 삭제해도 무방.
-- 따라서 login_id 보존용 호환 마이그레이션 없이, 레거시 행을 모두 비우고 users 테이블을
-- 이메일 식별 기준으로 깔끔히 재정의한다.
--
-- 기준선(B안 격리)은 불변: 계정 식별자만 username→email로 바뀔 뿐, 게임파일 격리/무전송
-- 로직은 그대로다. R2(wie-data)에 남는 고아 객체는 user_files 행이 사라져 접근 경로가 없고
-- (다운로드는 소유자 user_files 행 필요), 용량 집계(usedBytes)도 user_files만 더하므로
-- 누구의 quota에도 잡히지 않는다 — 무해하며, 정리는 선택(사용자가 R2에서 비우면 됨, S8).

PRAGMA foreign_keys = ON;

-- ── 1) 레거시 데이터 전부 삭제 (자식 → 부모 순서) ──────────────────────────────
DELETE FROM email_tokens;
DELETE FROM file_reports;
DELETE FROM user_files;
DELETE FROM inquiries;
DELETE FROM devices;
DELETE FROM saves;
DELETE FROM sessions;
DELETE FROM users;

-- ── 2) users 재정의: email = 유일 식별자 (login_id 제거) ────────────────────────
-- login_id 가 인라인 UNIQUE(암묵 인덱스)라 DROP COLUMN 이 막히므로, 빈 테이블을
-- 깔끔히 재생성한다. 자식 테이블의 FK 는 users(id) 를 이름으로 참조하므로, 동일한 id PK
-- 로 재생성하면 그대로 유효하다. (id 값/스키마는 보존, login_id 만 사라짐.)
DROP TABLE users;
CREATE TABLE users (
    id              TEXT    PRIMARY KEY,            -- uuid v4
    email           TEXT    NOT NULL UNIQUE,        -- 로그인 식별자 = 이메일
    email_verified  INTEGER NOT NULL DEFAULT 0,
    password_algo   TEXT    NOT NULL,               -- 'pbkdf2-sha256'
    password_iter   INTEGER NOT NULL,               -- 100k
    password_salt   TEXT    NOT NULL,               -- base64
    password_hash   TEXT    NOT NULL,               -- base64
    status          TEXT    NOT NULL DEFAULT 'active',
    strikes         INTEGER NOT NULL DEFAULT 0,     -- repeat-infringer (0003)
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email);
