# simple-stats 仕様

## 概要
zellij/tmuxのステータスラインに表示するための軽量システム情報収集CLIツール。

## 取得メトリクス
- **CPU使用率** — sysinfo経由。200msサンプリングで計測
- **メモリ** — 使用量/総量/使用率/スワップ（sysinfo経由）
- **ネットワーク** — 受信/送信速度（sysinfo経由、キャッシュベースデルタ計算）
- **GPU** — 温度/使用率/VRAM/モデル名（NVML経由、featureフラグ `gpu` で有効化）

## フォーマットシステム
`--format` オプションでテンプレート文字列を指定。

### プレースホルダー
| プレースホルダー | 説明 | 単位 |
|---|---|---|
| `{cpu}` | CPU使用率 | % |
| `{mem_used}` | メモリ使用量 | GiB |
| `{mem_total}` | メモリ総量 | GiB |
| `{mem_percent}` | メモリ使用率 | % |
| `{swap_used}` | スワップ使用量 | GiB |
| `{swap_total}` | スワップ総量 | GiB |
| `{net_down}` | ダウンロード速度 | MB/s |
| `{net_up}` | アップロード速度 | MB/s |
| `{gpu_temp}` | GPU温度 | ℃ |
| `{gpu_util}` | GPU使用率 | % |
| `{gpu_mem_used}` | VRAM使用量 | GiB |
| `{gpu_mem_total}` | VRAM総量 | GiB |
| `{gpu_name}` | GPUモデル名 | 文字列 |

### 精度指定
`{mem_used:.2}` のように `:.N` で小数桁数を制御可能。

## ビルドターゲット
| ターゲット | 用途 |
|---|---|
| `aarch64-apple-darwin` | macOS (ローカル開発) |
| `aarch64-unknown-linux-gnu` | Linux aarch64 |
| `x86_64-unknown-linux-gnu` | Linux amd64 |

## パフォーマンス特性
- CPU不使用時: ~6ms
- CPU使用時: ~200ms（サンプリング待ち）
- ネットワーク速度はキャッシュベースのデルタ計算（初回はN/A）
