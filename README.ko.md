# DustFril

DustFril은 개발 산출물을 스캔, 분석, 점검, 정리하기 위한 워크스페이스입니다.

이 저장소는 재사용 가능한 Rust 코어 크레이트, CLI 앱, Tauri 데스크톱 앱으로 나뉘어 있습니다.

## 워크스페이스 구성

- `crates/dustfril-core`: 스캔, 분석, 정리, 점검 로직을 담은 공용 코어
- `apps/dustfril-cli`: `dfr` 커맨드라인 인터페이스
- `apps/dustfril-tauri`: React + Tauri 기반 데스크톱 앱 셸
- `apps/dustfril-tauri/src-tauri`: `dustfril-core`와 연결된 Tauri Rust 백엔드

## 현재 기능

- Rust, Node.js, Java 워크스페이스의 정리 가능한 산출물 스캔
- 산출물 크기, 경과 일수, 정리 권장 상태 분석
- 실제 삭제 전 정리 계획 미리 보기
- 휴지통 이동 또는 영구 삭제 모드로 정리 실행
- `preinstall`, `postinstall` 같은 Node 라이프사이클 스크립트 점검
- 스캔·정리 활동 이력을 버전 형식으로 운영체제 앱 데이터 디렉터리에 저장
- Node 및 Rust 프로젝트의 오프라인 공급망 보안 점검
- 매니페스트와 lockfile에서 Node 및 Rust 의존성 inventory를 결정적으로 계산
- 지원 lockfile의 존재 여부와 Git 상태 점검
- 대상 개발 도구를 실행하지 않고 로컬 SHA-256 baseline과 실행 파일 무결성 비교
- CLI 정리 이력을 운영체제 앱 데이터 디렉터리에 저장

공급망 보안 스캐너는 v0.0.1 이후 작업입니다. v0.0.1 릴리스 범위는
`AGENTS.md`에 정의된 데스크톱 산출물 정리 흐름으로 유지됩니다.

## 현재 탐지 대상

| 생태계 | 탐지 대상 |
| ------ | --------- |
| Rust   | `target/` |
| Node.js | `node_modules/` |
| Java   | `build/` |

보안 스캐너가 구조적으로 분석하는 형식은 `package-lock.json`(1–3), pnpm
YAML, Bun JSONC `bun.lock`(1–2), Cargo.lock(1–4)입니다. Yarn lockfile과
레거시 바이너리 `bun.lockb`는 파싱하지 않으며, 이 파일들만 있는 경우 npm
lockfile 누락으로 잘못 보고하지 않습니다. Core API는 `Missing`, `Modified`,
`Untracked`, `Clean` 상태를 반환합니다. Git worktree에서는 porcelain과 같은
상태를 사용하고, Git 저장소가 아니면 파일 존재 여부만 확인합니다.

## CLI 사용법

워크스페이스 루트에서 다음처럼 실행합니다.

```bash
cargo run -p dustfril-cli -- <command>
```

예시:

```bash
cargo run -p dustfril-cli -- scan
cargo run -p dustfril-cli -- analyze
cargo run -p dustfril-cli -- clean --dry-run
cargo run -p dustfril-cli -- clean
cargo run -p dustfril-cli -- clean --permanent
cargo run -p dustfril-cli -- audit --node
cargo run -p dustfril-cli -- dependencies --node
cargo run -p dustfril-cli -- security scan --node
cargo run -p dustfril-cli -- integrity scan --tool node --tool git
```

생태계 필터나 대상 경로를 함께 지정할 수 있습니다.

```bash
cargo run -p dustfril-cli -- scan . --rust
cargo run -p dustfril-cli -- analyze /path/to/workspace --node
```

지원 명령:

- `scan [path] [--rust] [--node] [--java]`
- `analyze [path] [--rust] [--node] [--java]`
- `clean [path] [--dry-run] [--permanent] [--rust] [--node] [--java]`
- `audit [path] [--node]`
- `dependencies [path] [--rust] [--node] [--java]`
- `security scan [path] [--node]`
- `integrity scan [--tool <name>]...`

`security scan`은 `package.json`, `Cargo.toml`, `package-lock.json`,
`pnpm-lock.yaml`, `bun.lock`, `Cargo.lock`을 읽기 전용으로 오프라인 점검합니다.
의심스러운 라이프사이클 스크립트, 공개 레지스트리 외부에서 가져오는 의존성,
내장된 과거 손상 패키지 목록, 누락되거나 변경된 lockfile을 경고합니다. 형식이
잘못되었거나 지원되지 않으면 해당 파일 경로를 포함한 오류를 반환합니다. 탐지된
명령이나 패키지 매니저를 실행하지 않고, 네트워크 및 프로젝트 파일 변경도 사용하지
않습니다.

`dependencies`는 매니페스트와 lockfile만 읽어 카테고리별 직접 의존성 수,
해결된 lockfile 노드 수, 형식이 보존하는 경우의 전이 의존성 수, 여러 버전으로
해결된 패키지를 출력합니다. `package.json`은 npm `package-lock.json`(1–3),
`pnpm-lock.yaml`(5–9), Bun JSONC `bun.lock`(1–2)을 지원하고, Rust는
`Cargo.toml`과 `Cargo.lock`(1–4)을 지원합니다. lockfile 누락과 Yarn, 레거시
`bun.lockb`, Java 및 지원하지 않는 package manager는 명시적인 상태로
출력합니다. 설치된 의존성의 디스크 크기나 취약점 점수는 계산하지 않습니다.

`integrity scan`은 PATH에서 요청한 개발 도구를 찾고 파일 메타데이터와 바이트를
읽어 SHA-256을 스트리밍 계산합니다. 대상 실행 파일을 절대 실행하지 않으며,
activity history와 분리된 버전 형식의 baseline을 저장합니다. macOS에서는 시스템
`codesign`으로 읽기 전용 서명 정보도 확인하고, Linux와 Windows에서는 서명 검증을
명시적으로 `Unsupported`로 보고합니다. 기본 대상은 `node`, `bun`, `cargo`,
`rustc`, `git`, `java`, `gradle`이고, `--tool`을 반복해 일부만 선택할 수 있습니다.
경로 또는 해시 변경은 무결성 변경으로 보고하며 악성 코드의 증거라고 단정하지
않습니다. 서명 결과도 소프트웨어 전체의 신뢰성 판정이 아닙니다.

## 데스크톱 앱

데스크톱 앱은 현재 아래 워크플로를 UI로 제공합니다.

- scan
- analyze
- cleanup plan
- cleanup execution
- lifecycle script audit

`apps/dustfril-tauri`에서 실행합니다.

```bash
npm install
npm run tauri dev
```

## 개발

워크스페이스 루트에서 Rust 테스트를 실행합니다.

```bash
cargo test
```

## 로드맵

- 더 많은 생태계별 캐시와 산출물 탐지기 추가
- 더 자세한 감사 결과와 대응 가이드
- 데스크톱 워크플로 확장
- 설정 파일과 고급 필터링 지원

## 라이선스

MIT License
