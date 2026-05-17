# katana-canvas-forge — UI 分離計画 抜粋

作成日: 2026-05-17  
canonical: [`katana/docs/architecture/ui-separation/detailed-design-and-tasks.md`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)

## このファイルの位置付け

本ファイルは KatanA ecosystem の **UI 分離構想 master** から `katana-canvas-forge` (KCF) 担当部分を抜粋したもの。task ID は master と同一。**master が単一情報源**。

## Repository の役割

`katana-canvas-forge` (KCF) は **transitional backend / compatibility runtime** として位置付ける。

- public subsystem owner は `katana-document-viewer::forge` (KDV) に移譲する。
- KCF 自身は短期的には diagram rendering + export の backend 実装として残す。長期的には public API を縮小し、KCF CLI も KDV `cli_api` への delegate に変える。
- KCF の品質ゲート (Mermaid / Draw.io / ZenUML / Playwright / runtime asset checksum / reference compare) は維持する。
- 将来的には KDR (`katana-diagram-renderer`) と KDV forge への責務分割が進む。

詳細: master [`1.5 katana-canvas-forge`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#15-katana-canvas-forge) と [`5.2.4 KCF integration`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#524-kcf-integration)

## 担当 Phase

- **Phase 2 (受け側のみ)**: KDV が KCF backend を呼び出すので、本 repo は **触らない**。KCF 既存利用者への影響ゼロを維持。
- **Phase 7**: KCF crate 側の public API を縮小、CLI を KDV delegate に切り替える (本 repo のメイン作業)

### Phase 2 と Phase 7 の役割分担

- **Phase 2** = KDV 側に KCF を呼ぶ facade を追加するだけ。**本 repo は触らない**。
- **Phase 7** = KCF 側の public API 縮小 + CLI delegate 化 + deprecation。**本 repo の作業**。

詳細は master [`P2 スコープ宣言`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#p2-スコープ宣言) / [`P7 スコープ宣言`](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md#p7-スコープ宣言)。

## Task list (master 抜粋)

### P7-A. Responsibility split

- [ ] P7-A-001: KCF rendering responsibilities を一覧化する。
- [ ] P7-A-002: KCF export responsibilities を一覧化する。
- [ ] P7-A-004: KCF と KDR の duplicated renderer types を一覧化する。
- [ ] P7-A-005: KDR を diagram rendering canonical として ADR に記録する。
- [ ] P7-A-006: KDV forge を export canonical として ADR に記録する。
- [ ] P7-A-007: KCF は compatibility runtime として ADR に記録する。

### P7-B. API migration (KCF 側作業)

前提: Phase 2 (P2-E) で KDV 側に KCF→KDV の内部変換が完成していること。Phase 7 ではその mapping を canonical schema として固定し、KCF 側の重複 public 表記を deprecated に倒す。

- [ ] P7-B-005: KCF export command の input shape を `ExportRequest` 互換に縮小し、ADR に invariants を記録する。
- [ ] P7-B-006: KCF CLI 各 subcommand を KDV `cli_api` delegate 実装に置き換える PR を作る。
- [ ] P7-B-007: KCF README に「Phase 2 以降は KDV facade 経由の利用を推奨」「KCF public API は transitional」と policy を追加する。
- [ ] P7-B-008: KCF release note に future deprecation timeline (target version, removal candidate) を記載する。

(P7-B-001 〜 P7-B-004 は KDV 側で ADR を書く task。本 repo は ADR の deprecation 条件を実装に反映する責務だけ持つ。)

### P7-C. Runtime assets (KCF 側で KDV 側へ移管支援)

- [ ] P7-C-006: KCF の runtime gate を KDV forge gate へ mirror する。
- [ ] P7-C-007: asset update command の ownership を整理する。

## 前提 (depends on) / 出力 (provides)

- **前提**:
  - Phase 2 (P2-E KCF backend integration in KDV) 完成
  - できれば Phase 5 (KatanA export migration) 完了 — 既存利用者影響を最小化するため
  - `katana-canvas-forge` を transitional backend とする ADR (P0-B-006)

- **出力**:
  - KCF public API 縮小版 + deprecated 表記
  - KCF CLI が KDV `cli_api` への薄い delegate
  - KCF future deprecation timeline (release note)
  - KCF runtime asset gate の KDV ミラー

## Done criteria

本 repo に関する master 9 章 Done criteria のうち、該当項目:

- [ ] KCF は backend / compatibility layer になる
- [ ] KatanA で export が KDV forge 経由で動く (= KCF 直接利用が消えている)
- [ ] KCF README に transitional backend / delegate policy が明記されている
- [ ] KCF CLI が KDV `cli_api` への delegate になっている

## drift 検出

- 本ファイルの task ID は master と完全一致する。
- P8-A-001 の CI script が master と本ファイルの task ID 一致を検査する。

## 参照リンク

- [master detailed-design-and-tasks.md](../../katana/docs/architecture/ui-separation/detailed-design-and-tasks.md)
- [master principles.md](../../katana/docs/architecture/ui-separation/principles.md)
- [overview README](../../katana/docs/architecture/ui-separation/README.md)
- [KDV repo の Phase 2 / 7 抜粋](../../katana-document-viewer/docs/ui-separation-plan.md)
- [KDR repo の Phase 7 抜粋](../../katana-diagram-renderer/docs/ui-separation-plan.md)
- [既存 docs/release.md](release.md)
- [既存 docs/runtime-assets.md](runtime-assets.md)
