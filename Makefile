CONFIG_FILE := kernel_conf.json
MACHINE_TRIPLE := $(shell rustc -vV | grep host | awk '{ print $$2 }')

.PHONY: build kernel run defconfig menuconfig clean info
build: kernel

defconfig:
	@cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- --generate-defconfig $(CONFIG_FILE)

# If kernel_conf is not found, generate it
ifeq ("$(wildcard $(CONFIG_FILE))","")
	$(MAKE) defconfig
endif
TARGET := $(shell jq -r '.target' kernel_conf.json)
TARGET_TRIPLE := "$(TARGET)-unknown-none"

info:
	@echo "Target triple: $(TARGET_TRIPLE)"
	@echo "Config file: $(CONFIG_FILE)"
	@echo "Machine triple: $(MACHINE_TRIPLE)"

# Generate default config if it doesn't exist
$(CONFIG_FILE):
	@cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- --generate-defconfig $(CONFIG_FILE)

kernel: $(CONFIG_FILE)
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" build

menuconfig:
	cargo run -p menuconfig --target "$(MACHINE_TRIPLE)" -- $(CONFIG_FILE)

run:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" run

test:
	@echo "Running tests..."
	@echo "TEST crate: volatile"
	@cargo test -p volatile --target "$(MACHINE_TRIPLE)" --features std
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" test

clippy:
	@$(MAKE) -C kernel TARGET_TRIPLE="$(TARGET_TRIPLE)" CONFIG_FILE="../$(CONFIG_FILE)" clippy

clean:
	@echo "Cleaning..."
	cargo clean
	rm -f kernel_conf.json
