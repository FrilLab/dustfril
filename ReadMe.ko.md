# DustFril

Rust 개발자를 위한 산출물(Artifact) 분석 및 정리 도구입니다.

DustFril은 Cargo와 Rust 도구들이 생성하는 빌드 산출물과 캐시를 탐지하고, 용량을 분석하며, 안전하게 정리할 수 있도록 돕는 것을 목표로 합니다.

Rust 프로젝트를 오래 관리하다 보면 `target/`, Cargo 캐시 등의 디렉터리가 수 GB에서 수십 GB까지 증가할 수 있습니다.

DustFril은 이러한 생성 파일들을 쉽고 안전하게 관리할 수 있는 CLI 도구를 지향합니다.

## 주요 기능

### 현재 목표

- Rust 산출물 탐지
- 디스크 사용량 분석
- Cargo 캐시 분석
- 안전한 정리 기능

### 지원 예정

- `target/`
- `~/.cargo/registry`
- `~/.cargo/git`

## 사용 예시

프로젝트 스캔:

```bash
dfr scan
```

용량 분석:

```bash
dfr analyze
```

삭제 예정 파일 확인:

```bash
dfr clean --dry-run
```

실제 정리:

```bash
dfr clean
```

## 로드맵

### Phase 1

- Cargo 프로젝트 탐지
- Rust 산출물 탐지
- 기본 CLI 구현

### Phase 2

- 디스크 사용량 분석
- Dry Run 지원
- 안전 삭제 기능

### Phase 3

- 인터랙티브 터미널 UI
- 설정 파일 지원
- 고급 필터링

### Phase 4

- 데스크톱 애플리케이션
- 다중 언어 생태계 지원

## 철학

DustFril은 다음 원칙을 중요하게 생각합니다.

- 안전성 우선
- 명시적 사용자 동작
- 투명한 동작
- 개발자 친화적 경험

DustFril은 사용자의 확인 없이 파일을 삭제하지 않습니다.

## 라이선스

MIT License
