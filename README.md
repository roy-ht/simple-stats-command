# simple-stats

zellij / tmux のステータスラインに表示するための、軽量なシステム情報収集CLIツール。

定期実行(数秒間隔)を前提に設計されており、**起動から出力完了まで約6ms**で動作する。

## 取得できる情報

- CPU使用率
- メモリ使用量 / 総量 / 使用率
- スワップ使用量 / 総量
- ネットワーク送受信速度
- NVIDIA GPU 温度 / 使用率 / VRAM / モデル名 / CUDAバージョン (オプション)

## インストール

### 必要なもの

- Rust toolchain (1.85+)
- クロスコンパイルする場合: [zig](https://ziglang.org/) + [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)

### ビルド

```bash
# macOS (GPUなし)
cargo build --release

# Linux (GPUなし)
cargo zigbuild --release --target aarch64-unknown-linux-gnu

# Linux (NVIDIA GPU対応)
cargo zigbuild --release --target aarch64-unknown-linux-gnu --features gpu
```

`make build-all` で macOS / Linux aarch64+GPU / Linux x86_64 の3ターゲットを一括ビルドできる。

## 使い方

```bash
# デフォルトフォーマット
simple-stats
# => 23% | 12.4/32.0G | 1.2/0.3M/s

# カスタムフォーマット
simple-stats --format "CPU:{cpu}% MEM:{mem_percent:.0}%"
# => CPU:23% MEM:39%

# GPU情報を含める (gpuフィーチャー有効ビルドが必要)
simple-stats --format "{cpu}% | {gpu_temp}C {gpu_util}% CUDA:{cuda_ver}"
# => 23% | 38C 0% CUDA:13.0

# 特定のネットワークインターフェースのみ監視
simple-stats --format "{net_down:.2}/{net_up:.2}M/s" -I eth0

# 情報取得時刻を表示
simple-stats --format "{cpu}% | {time}"
# => 23% | 14:23:05
```

### オプション

| オプション | 説明 | デフォルト |
|---|---|---|
| `-f`, `--format` | 出力フォーマットテンプレート | `{cpu}% \| {mem_used:.1}/{mem_total:.1}G \| {net_down:.1}/{net_up:.1}M/s` |
| `-I`, `--interface` | 監視対象のネットワークインターフェース | 全インターフェース合算 |
| `--cache-path` | キャッシュファイルのパス | `/tmp/simple-stats.bin` |

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
| `{gpu_temp}` | GPU温度 | C |
| `{gpu_util}` | GPU使用率 | % |
| `{gpu_mem_used}` | VRAM使用量 | GiB |
| `{gpu_mem_total}` | VRAM総量 | GiB |
| `{gpu_name}` | GPUモデル名 | - |
| `{cuda_ver}` | CUDAバージョン | - |
| `{time}` | 情報取得時刻 (ローカルタイム `HH:MM:SS`) | - |

`{mem_used:.2}` のように `:.N` を付けると小数点以下の桁数を指定できる。

### 初回実行時の挙動

CPU使用率とネットワーク速度は前回との差分で算出するため、初回実行時は `N/A` と表示される。2回目以降から正常な値が表示される。

## zellij / tmux での使用例

### zellij (zjstatus)

```kdl
widget "stats" {
    command "simple-stats"
    args "--format" " {cpu}% | {mem_used:.1}/{mem_total:.1}G | {net_down:.1}/{net_up:.1}M/s "
    interval 5
}
```

### tmux

```bash
set -g status-right '#(simple-stats --format "CPU:{cpu}%% MEM:{mem_percent:.0}%%")'
set -g status-interval 5
```

## アーキテクチャ

```
simple-stats
  |-- cli.rs           手動引数パース (依存クレートなし)
  |-- cache.rs         40バイト固定長バイナリキャッシュ
  |-- clock.rs         ローカル時刻取得 (localtime_r FFI)
  |-- format.rs        テンプレートエンジン
  |-- metrics/
  |     |-- cpu.rs     Linux: /proc/stat 直読み / macOS: sysinfo
  |     |-- memory.rs  sysinfo経由
  |     |-- network.rs sysinfo経由 + キャッシュデルタ
  |     |-- gpu.rs     NVML経由 (featureフラグ)
  |-- main.rs          エントリポイント
```

### 設計方針

- **外部コマンド不使用** -- `nvidia-smi` 等は呼ばない。すべてライブラリ/OS API経由
- **必要なメトリクスのみ収集** -- フォーマット文字列を解析し、使わないサブシステムはスキップ
- **キャッシュベースのデルタ計算** -- CPU・ネットワークは前回値との差分で算出。sleep不要
- **GPU はオプション** -- `gpu` featureフラグで分離。NVMLがない環境でもビルド・動作可能
- **最小依存** -- ランタイム依存は `sysinfo` のみ (GPU有効時は `nvml-wrapper` 追加)

## ビルドターゲット

| ターゲット | コマンド | バイナリサイズ |
|---|---|---|
| macOS (arm64) | `cargo build --release` | ~350KB |
| Linux aarch64 + GPU | `cargo zigbuild --release --target aarch64-unknown-linux-gnu --features gpu` | ~450KB |
| Linux x86_64 | `cargo zigbuild --release --target x86_64-unknown-linux-gnu` | ~390KB |

## ライセンス

MIT
