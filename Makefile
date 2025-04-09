TARGET_TRIPLE := x86_64-unknown-none
CONFIG_FILE := kernel_conf.json

.PHONY: build kernel run defconfig menuconfig clean
build: kernel

# Generate default config if it doesn't exist
$(CONFIG_FILE):
	@cargo run -p menuconfig -- --generate-defconfig $(CONFIG_FILE)

kernel: $(CONFIG_FILE)
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" build

defconfig:
	@cargo run -p menuconfig -- --generate-defconfig $(CONFIG_FILE)

menuconfig:
	cargo run -p menuconfig -- $(CONFIG_FILE)

run:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" run

test:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" test

clippy:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" clippy

clean:
	cargo clean
	rm -f kernel_conf.json
