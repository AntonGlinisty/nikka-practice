QEMU_OPTIONS = -cpu qemu64 -m size=50M -smp cpus=4 -device isa-debug-exit,iobase=0xF4,iosize=0x04
QEMU_GTK = $(QEMU_OPTIONS) -display gtk -serial file:serial.out
QEMU_NOX = $(QEMU_OPTIONS) -nographic -serial mon:stdio
QEMU_CURSES = $(QEMU_OPTIONS) -display curses -serial file:serial.out

QUIT_MESSAGE = "To quit QEMU type Ctrl-A then X. For QEMU console type Ctrl-A then C.\n"
WAIT_MESSAGE = "Will wait for GDB to attach. Run 'make gdb' in a separate console after qemu starts.\n"

BUILD_MODE = debug

ifeq ($(BUILD_MODE),debug)
    BUILD_MODE_FLAG =
else
    BUILD_MODE_FLAG = --release
endif

RUST_PATH = $(HOME)/.cargo/bin

export PATH := $(RUST_PATH):$(PATH)

CRATES = ku serial text kernel sem

all: test run

run: install
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK); \
	)

run-gdb: install
	@echo $(WAIT_MESSAGE)
	@echo
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_GTK) -gdb tcp::1234 -S; \
	)

nox: install
	@echo $(QUIT_MESSAGE)
	@echo
	@( \
		cd kernel; \
		rm --force serial.out; \
		cargo run $(BUILD_MODE_FLAG) -- $(QEMU_NOX); \
	)

nox-gdb: install
	@echo $(QUIT_MESSAGE)
	@echo $(WAIT_MESSAGE)
	@echo
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
		rustup component add clippy; \
		rustup component add miri; \
		rustup component add rustfmt; \
		for crate in $(CRATES); do \
			cd $$crate; \
			if [ .$$crate = .ku ]; then \
				test_flags='-- --test-threads=1'; \
			else \
				test_flags=''; \
			fi; \
			RUST_BACKTRACE=1 cargo test $$test_flags || exit 1; \
			cargo clippy || exit 1; \
			cargo fmt --check || exit 1; \
			cd - >/dev/null; \
		done; \
		cd ku; \
		for test in 1-time-2-once-lock 1-time-5-correlation-point; do \
			MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --test $$test; \
		done; \
		cd - >/dev/null; \
		cd sem/rwlock; \
		MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test -- --nocapture --test-threads=1; \
		cd - >/dev/null; \
	)

test-gdb: install
	@( \
		for crate in kernel; do \
			cd $$crate; \
			RUST_BACKTRACE=1 cargo test -- -gdb tcp::1234 -S; \
			cd - >/dev/null; \
		done; \
	)

lab: $(RUST_PATH)/mdbook doc lab/src/*.dot lab/src/*.md lab/src/*.puml
	@( \
		if grep --quiet 'compose::begin_private' ku/src/ipc/pipe.rs; then \
			cd ku; \
			cargo test --test pipe-visualization; \
			cd - >/dev/null; \
			cd lab/src; \
			sed '/(node0)/{s@headerwritten@headerclear@g;s@01@00@g;}' < 6-um-1-pipe-8-write-tx-1-commit.tex > 6-um-1-pipe-8-write-tx-1-commit-1-part.tex; \
			sed '/(node0)/{s@headerread@headerwritten@g;s@02@01@g;}' < 6-um-1-pipe-19-read-tx-commit.tex > 6-um-1-pipe-19-read-tx-commit-1-part.tex; \
			cd - >/dev/null; \
		fi; \
		sed -i 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' lab/src/*.md; \
		cd lab/src; \
		for i in *.dot; do dot -Tsvg $$i > $${i%dot}svg; done; \
		for i in *.puml; do java -jar /usr/share/java/plantuml/plantuml.jar -tsvg $$i; done; \
		for i in 6-um-1-pipe*.tex; do \
			sed -i 's@\s*$$@@g;' $$i; \
			pdflatex -halt-on-error $$i >/dev/null; \
			latex_error=$$?; \
			cat $${i%tex}log | grep --after-context=2 '^!'; \
			if [ $$latex_error -ne 0 ]; then exit $$latex_error; fi; \
			pdf2svg $${i%tex}pdf $${i%tex}svg; \
		done; \
		cd - >/dev/null; \
		cd lab; \
		cp book-base.toml book.toml; \
		mdbook build; \
		cp book-linkcheck.toml book.toml; \
		mdbook build; \
		mv book/html html; \
		rm --force --recursive book book.toml; \
		mkdir book; \
		mv html book/; \
		mv book/html/* book/; \
		cd - >/dev/null; \
		rm --force lab/src/*.aux lab/src/*.log lab/src/*.pdf; \
	)

fs-dump:
	@xxd fs.img | sed 's@.*0000 0000 0000 0000 0000 0000 0000 0000.*@...@' | uniq

doc: FORCE
	@( \
		cargo doc --document-private-items; \
		rm --force $$(grep --files-with-matches 'compose::begin_private' $$(find target/doc/src -name \*.rs.html) ); \
		rm --force --recursive doc; \
		mv target/doc .; \
	)

clean:
	@( \
		for crate in $(CRATES) tools; do \
			cd $$crate; \
			cargo clean; \
			cd - >/dev/null; \
		done; \
	)
	@rm --force --recursive \
		doc \
		fs.img \
		kernel/serial.out \
		ku/*.aux ku/*.log ku/*.pdf ku/*.tex \
		lab/book \
		lab/src/*.aux lab/src/*.log lab/src/*.pdf lab/src/*.svg \
		$(find . -name \*.rs.bk)

stat:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --stat

compose:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --out-path .

install:
	@(which rustup > /dev/null || (curl https://sh.rustup.rs -sSf | sh))
	@(which bootimage > /dev/null || (cargo install bootimage))
	@(which rustfilt > /dev/null || (cargo install rustfilt))
	@(which cargo-expand > /dev/null || (cargo install cargo-expand))

$(RUST_PATH)/mdbook:
	cargo install mdbook
	cargo install mdbook-mermaid
	cargo install mdbook-plantuml

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
