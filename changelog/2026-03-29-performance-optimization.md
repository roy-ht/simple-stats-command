# パフォーマンス最適化 (v0.2.0)

## 変更
- CPU計測: 200msスリープ廃止。Linux: `/proc/stat`直読み+キャッシュデルタ、macOS: sysinfo単発refresh
- CLIパーサー: clap → 手動パース（依存クレート削減）
- キャッシュ形式: JSON → 固定長40バイトのバイナリ（serde/serde_json依存削除）

## 削除した依存
- `clap` (4.x)
- `serde` (1.x)
- `serde_json` (1.x)

## 効果
- バイナリサイズ: 524KB → 346KB (-34%)
- 実行時間(CPU込み): 800ms → 6ms (-99%)
- ランタイム依存: sysinfoのみ（GPU有効時はnvml-wrapper追加）
