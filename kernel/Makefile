TARGET_TRIPLE ?= x86_64-unknown-none
CONFIG_FILE ?= ../kernel_conf.json

DEBUG := $(shell jq -r '.debug' $(CONFIG_FILE))
RUSTFLAGS := --target $(TARGET_TRIPLE)
RUSTENV := CONFIG_FILE="$(CONFIG_FILE)"

ifeq ($(DEBUG), false)
	RUSTFLAGS += --release
endif

.PHONY: build run test clippy

build:
	cargo build -p hadron-kernel $(RUSTFLAGS)

run:
	cargo run -p hadron-kernel $(RUSTFLAGS)

test:
	@echo "TEST crate: hadron-kernel"
	@cargo test -p hadron-kernel $(RUSTFLAGS)

clippy:
	cargo clippy -p hadron-kernel $(RUSTFLAGS)
