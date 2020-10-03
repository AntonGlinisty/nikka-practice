QEMU_OPTIONS = -cpu qemu64 -m size=5M
QEMU_GTK = $(QEMU_OPTIONS) -display gtk -serial file:serial.out
QEMU_NOX = $(QEMU_OPTIONS) -nographic -serial mon:stdio
QEMU_CURSES = $(QEMU_OPTIONS) -display curses -serial file:serial.out

QUIT_MESSAGE = "To quit QEMU type Ctrl-A then X. For QEMU console type Ctrl-A then C.\n"
WAIT_MESSAGE = "Waiting for GDB to attach. Run 'make gdb' in a separate console.\n"

BUILD_MODE = debug

ifeq ($(BUILD_MODE),debug)
    BUILD_MODE_FLAG =
else
    BUILD_MODE_FLAG = --release
endif

RUST_PATH = $(HOME)/.cargo/bin

export PATH := $(RUST_PATH):$(PATH)

CRATES = serial text kernel

all: test run

run: bootimage
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK); \
	)

run-gdb: bootimage
	@echo $(WAIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK) -gdb tcp::1234 -S; \
	)

nox: bootimage
	@echo $(QUIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_NOX); \
	)

nox-gdb: bootimage
	@echo $(QUIT_MESSAGE)
	@echo $(WAIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_NOX) -gdb tcp::1234 -S; \
	)

curses: bootimage
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_CURSES); \
	)

kill:
	@kill $$(ps -u $$USER -o pid,comm | awk '$$2 == "qemu-system-x86" { print $$1; }')

gdb:
	gdb -command=gdbinit

FORCE:

test: $(RUST_PATH)/cargo
	@( \
		cd text; \
		RUST_BACKTRACE=1 cargo test; \
	)

fmt: $(RUST_PATH)/rustfmt
	@find . -name \*.rs -a ! -path \*/target/\* -exec rustfmt \{\} \;

clean: $(RUST_PATH)/cargo
	@( \
		for crate in $(CRATES); do \
			cd $$crate; \
			cargo clean; \
			cd - >/dev/null; \
		done; \
	)
	@rm --force \
		kernel/serial.out \
		$(find . -name \*.rs.bk)

install: FORCE $(RUST_PATH)/rustup

update: FORCE install
	@rustup update
	@( \
		for crate in $(CRATES); do \
			cd $$crate; \
			cargo update; \
			cd - >/dev/null; \
		done; \
	)

$(RUST_PATH)/rustup:
	@curl https://sh.rustup.rs -sSf | sh
	@rustup install $$(grep -o 'nightly-[^"]*' rust-toolchain.toml)
	@rustup component add rust-src

$(RUST_PATH)/cargo: $(RUST_PATH)/rustup

$(RUST_PATH)/cargo-xbuild: $(RUST_PATH)/cargo
	@cargo install --force cargo-xbuild

$(RUST_PATH)/rustfmt: $(RUST_PATH)/cargo
	@cargo install --force rustfmt

bootimage: $(RUST_PATH)/bootimage llvm-tools-preview

$(RUST_PATH)/bootimage: $(RUST_PATH)/cargo
	@cargo install --force bootimage

llvm-tools-preview: FORCE
	@rustup component add llvm-tools-preview

nikka.pdf: FORCE
	@trueprint \
		--no-page-break-after-function \
		--point-size=16 \
		--no-holepunch \
		--no-top-holepunch \
		--single-sided \
		--one-up \
		--no-cover-sheet \
		--function-index \
		--landscape \
		--no-headers \
		--output=/dev/stdout \
		$$(jj file list | grep '\.rs$$' | sort --unique) | \
		ps2pdf /dev/stdin $@
