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

## 현재 탐지 대상

| 생태계 | 탐지 대상 |
| ------ | --------- |
| Rust   | `target/` |
| Node.js | `node_modules/` |
| Java   | `build/` |

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
