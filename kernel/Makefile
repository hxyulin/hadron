TARGET_FILE ?= ../targets/x86_64-unknown-hadron.json
CONFIG_FILE ?= ../kernel_conf.json

DEBUG := $(shell jq -r '.debug' $(CONFIG_FILE))
RUSTFLAGS := -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem --target $(TARGET_FILE)
RUSTENV := CONFIG_FILE="$(CONFIG_FILE)"

ifeq ($(DEBUG), false)
	RUSTFLAGS += --release
endif

.PHONY: build run test clippy

build:
	RUSTFLAGS=$(RUSTCFLAGS) cargo build -p hadron-kernel $(RUSTFLAGS)

run:
	RUSTFLAGS=$(RUSTCFLAGS) cargo run -p hadron-kernel $(RUSTFLAGS)

test:
	@echo "TEST crate: hadron-kernel"
	@cargo test -p hadron-kernel $(RUSTFLAGS)

clippy:
	cargo clippy -p hadron-kernel $(RUSTFLAGS)
