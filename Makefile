.PHONY: build-local build-darwin-aarch64 build-linux-aarch64 build-linux-x86_64 build-all test clean

# ローカル (macOS) ビルド - GPUなし
build-local:
	cargo build --release

# macOS aarch64 (Apple Silicon) ビルド
build-darwin-aarch64:
	cargo build --release --target aarch64-apple-darwin
	mkdir -p bin
	cp target/aarch64-apple-darwin/release/simple-stats bin/simple-stats-darwin-aarch64

# aarch64 Linux ビルド
build-linux-aarch64:
	cargo zigbuild --release --target aarch64-unknown-linux-gnu --features gpu
	mkdir -p bin
	cp target/aarch64-unknown-linux-gnu/release/simple-stats bin/simple-stats-linux-aarch64

# x86_64 Linux ビルド
build-linux-x86_64:
	cargo zigbuild --release --target x86_64-unknown-linux-gnu --features gpu
	mkdir -p bin
	cp target/x86_64-unknown-linux-gnu/release/simple-stats bin/simple-stats-linux-x86_64

# 全ターゲットビルド
build-all: build-darwin-aarch64 build-linux-aarch64 build-linux-x86_64

test:
	cargo test

clean:
	cargo clean
