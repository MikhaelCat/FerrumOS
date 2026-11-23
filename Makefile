.PHONY: all build test clean build-mips

all: build

build:
	@echo "Building FerrumOS Core (Host)..."
	cd core && cargo build --release

test:
	@echo "Running FerrumOS Core unit tests..."
	cd core && cargo test

build-mips:
	@echo "Cross-compiling FerrumOS Core for MediaTek MT7620 (MIPS SoftFloat)..."
	bash tools/build_mips.sh

clean:
	cd core && cargo clean
	rm -rf bin
