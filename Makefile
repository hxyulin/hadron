.PHONY: build run clean menuconfig test

build:
	cargo run -p buildscript -- build

run:
	cargo run -p buildscript -- run

clean:
	cargo run -p buildscript -- clean

menuconfig:
	cargo run -p buildscript -- menuconfig

test:
	cargo run -p buildscript -- test
