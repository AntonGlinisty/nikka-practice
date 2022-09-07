QEMU_OPTIONS = -cpu qemu64 -m size=50M -smp cpus=4 -device isa-debug-exit,iobase=0xF4,iosize=0x04
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

CRATES = ku serial text kernel

all: test run

run: install
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK); \
	)

run-gdb: install
	@echo $(WAIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK) -gdb tcp::1234 -S; \
	)

nox: install
	@echo $(QUIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_NOX); \
	)

nox-gdb: install
	@echo $(QUIT_MESSAGE)
	@echo $(WAIT_MESSAGE)
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_NOX) -gdb tcp::1234 -S; \
	)

curses: install
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

test: install
	@( \
		for crate in $(CRATES); do \
			cd $$crate; \
			RUST_BACKTRACE=1 cargo test || exit 1; \
			cd - >/dev/null; \
		done; \
	)

test-gdb: install
	@( \
		for crate in kernel; do \
			cd $$crate; \
			RUST_BACKTRACE=1 cargo test -- -gdb tcp::1234 -S; \
			cd - >/dev/null; \
		done; \
	)

lab: $(RUST_PATH)/mdbook doc lab/src/*.dot lab/src/*.md
	@( \
		cd lab/src; \
		for i in *.dot; do dot -Tsvg $$i > $${i%dot}svg; done; \
		cd - >/dev/null; \
		cd lab; \
		mdbook build; \
		mv book/html html; \
		rm --force --recursive book; \
		mkdir book; \
		mv html book/; \
		mv book/html/* book/; \
		cd - >/dev/null; \
	)

doc: FORCE
	@( \
		cargo doc --document-private-items; \
		find target/doc/src -name \*.rs.html -delete; \
		rm --force --recursive doc; \
		mv target/doc .; \
	)

clean:
	@( \
		for crate in $(CRATES); do \
			cd $$crate; \
			cargo clean; \
			cd - >/dev/null; \
		done; \
	)
	@rm --force --recursive \
		doc \
		kernel/serial.out \
		lab/book \
		lab/src/*.svg \
		$(find . -name \*.rs.bk)

stat:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --stat

compose:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --out-path .

install:
	@(which rustup > /dev/null || (curl https://sh.rustup.rs -sSf | sh))
	@(which bootimage > /dev/null || (cargo install bootimage))

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
