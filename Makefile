.PHONY: build-local build-linux-aarch64-gpu build-linux-x86_64 build-all test clean

# ローカル (macOS) ビルド - GPUなし
build-local:
	cargo build --release

# aarch64 Linux ビルド
build-linux-aarch64-gpu:
	cargo zigbuild --release --target aarch64-unknown-linux-gnu --features gpu

# x86_64 Linux ビルド
build-linux-x86_64:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu --features gpu

# 全ターゲットビルド
build-all: build-local build-linux-aarch64-gpu build-linux-x86_64

test:
	cargo test

clean:
	cargo clean
