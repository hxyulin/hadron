CONFIG_FILE := kernel_conf.json
MACHINE_TRIPLE := $(shell rustc -vV | grep host | awk '{ print $$2 }')
TARGET := $(shell jq -r '.target' kernel_conf.json)
TARGET_TRIPLE := "$(TARGET)-unknown-none"

.PHONY: build kernel run defconfig menuconfig clean info
build: kernel

info:
	@echo "Target triple: $(TARGET_TRIPLE)"
	@echo "Config file: $(CONFIG_FILE)"
	@echo "Machine triple: $(MACHINE_TRIPLE)"

# Generate default config if it doesn't exist
$(CONFIG_FILE):
	@cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- --generate-defconfig $(CONFIG_FILE)

kernel: $(CONFIG_FILE)
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" build

defconfig:
	@cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- --generate-defconfig $(CONFIG_FILE)

menuconfig:
	cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- $(CONFIG_FILE)

run:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" run

test:
	@echo "Running tests..."
	@echo "TEST crate: volatile"
	@cargo test -p volatile --target "$(MACHINE_TRIPLE)" --features std
	@echo "TEST crate: arch-x86_64"
	@RUSTFLAGS="-C link-arg=-Tutil/limine-x86_64-link.ld -C relocation-model=static" cargo test -p arch-x86_64 --target x86_64-unknown-none
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" test

clippy:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" clippy

clean:
	@echo "Cleaning..."
	cargo clean
	rm -f kernel_conf.json
