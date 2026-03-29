# 初期実装

## 追加
- CPU使用率、メモリ、ネットワーク通信量、GPU情報の収集機能
- テンプレートベースのフォーマットシステム（`--format`オプション）
- ネットワークインターフェース指定（`--interface`オプション）
- キャッシュベースのデルタ計算（ネットワーク速度算出用）
- GPU対応（NVML経由、`gpu` featureフラグで有効化）
- マルチターゲットビルド（macOS, Linux aarch64, Linux x86_64）
- cargo-zigbuildによるクロスコンパイル対応

## 技術的判断
- CPU使用率は200msサンプリング方式（sysinfo仕様上2回のrefreshが必要）
- GPUの各フィールドを個別にOption化（デバイスによって非対応APIがある）
- リリースビルドはLTO + strip + panic=abortで最適化
