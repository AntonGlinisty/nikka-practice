QEMU_OPTIONS = -cpu qemu64 -m size=50M -smp cpus=4 -device isa-debug-exit,iobase=0xF4,iosize=0x04
QEMU_GTK = $(QEMU_OPTIONS) -display gtk -serial file:serial.out
QEMU_NOX = $(QEMU_OPTIONS) -nographic -serial mon:stdio
QEMU_CURSES = $(QEMU_OPTIONS) -display curses -serial file:serial.out

PLANTUML=$(shell ls /usr/share/java/plantuml/plantuml.jar /usr/share/plantuml/plantuml.jar 2>/dev/null | head --lines 1 || true)

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

CRATES = ku pci pic8259 serial text kernel sem
WORKSPACES = . sem

LECTURES_MD=$(shell ls lectures/*.md)
LECTURES_PDF=$(LECTURES_MD:.md=.pdf)

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

lint: install
	@( \
		rustup component add clippy; \
		rustup component add rustfmt; \
		nikka_root=$$(pwd); \
		for crate in $(CRATES); do \
			cd $$crate; \
			cargo spellcheck --cfg $$nikka_root/tools/lint/spellcheck.toml --code 1; \
			typos --config $$nikka_root/tools/lint/typos.toml || exit 1; \
			cargo fmt --check || exit 1; \
			cargo clippy --all-targets || exit 1; \
			cd - >/dev/null; \
		done; \
		taplo format --config tools/lint/taplo.toml --check $$(find . -name \*.toml) || exit 1; \
		long_lines=$$(grep '^.\{101\}' $$(find . -name \*.rs | grep -v -e '^\./target/' -e '^\./sem/target/' -e 'tests/with_sentinel_frame/expand') | grep -v -e '^[^:]*: *///' -e 'compose::public' -e 'https*://'); \
		if [ ! -z "$$long_lines" ]; then \
			echo "Too long lines found."; \
			echo "$$long_lines"; \
			exit 1; \
		fi; \
		typos=$$(cat lab/src/*.md sem/*/*.md | sed 's@\(https://[w.]*youtube.com/[a-z?=]*\)[0-9A-Za-z_-]*@\1@g' | hunspell -d ru_RU,en_US -p tools/lint/spellcheck.exceptions -l | sort -u); \
		if [ ! -z "$$typos" ]; then \
			echo "Typos found."; \
			echo "$$typos"; \
			exit 1; \
		fi; \
	)

test: install
	@( \
		rustup component add miri; \
		for crate in $(CRATES); do \
			cd $$crate; \
			if [ .$$crate = .ku ]; then \
				test_flags='--profile ci -- --test-threads=1'; \
			elif [ .$$crate = .kernel ]; then \
				test_flags='--profile ci --features forbid-leaks'; \
			else \
				test_flags=''; \
			fi; \
			RUST_BACKTRACE=1 cargo test $$test_flags || exit 1; \
			cd - >/dev/null; \
		done; \
		cd ku; \
		for test in 1-time-2-once-lock 1-time-5-correlation-point; do \
			MIRIFLAGS="-Zmiri-disable-isolation" cargo miri test --test $$test; \
		done; \
		cargo test --profile benchmark --test 3-allocator-5-small-memory-allocator --features benchmark -- --test-threads=1; \
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

lab: $(RUST_PATH)/mdbook lab/src/*.dot lab/src/*.md lab/src/*.puml FORCE
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
		sed -i 's@https://www.1024cores.net/.../nikka/bounded-mpmc-queue@https://www.1024cores.net/home/lock-free-algorithms/queues/bounded-mpmc-queue@g' lab/src/*.md; \
		cargo run --manifest-path tools/check/Cargo.toml -- --student-repo . --original-repo . --ci-branch-name submit/- --user-id - --dump-dependencies > lab/src/0-intro-1-nikka-lab-dependencies.dot; \
		for i in 1-hw "2-memory 2-mm" 3-allocator 4-process 5-sync "6-user-memory-tricks 6-um" "7-file-system 7-fs" "8-queues 8-queues"; do \
			lab_dot=$$(echo "$$i $$i"); \
			lab=$$(echo "$$lab_dot" | cut -f1 -d' '); \
			dot=$$(echo "$$lab_dot" | cut -f2 -d' '); \
			cargo run --manifest-path tools/check/Cargo.toml -- \
				--student-repo . \
				--original-repo . \
				--ci-branch-name submit/- \
				--user-id - \
				--dump-group-dependencies lab-$${lab} > lab/src/$${dot}-0-intro-lab-dependencies.dot; \
		done; \
		cd lab/src; \
		for i in *.dot; do dot -Tsvg $$i > $${i%dot}svg; done; \
		for i in *.puml; do java -jar $(PLANTUML) -tsvg $$i; done; \
		for i in *.svg.criterion; do cp $$i $${i%.criterion}; done; \
		for i in 3-allocator-7-*.tex 6-um-1-pipe*.tex; do \
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
		rm --force book.toml; \
	)
#		cp book-linkcheck.toml book.toml; \
#		mdbook build; \
#		mv book/html html; \
#		rm --force --recursive book book.toml; \
#		mkdir book; \
#		mv html book/; \
#		mv book/html/* book/; \
#		cd - >/dev/null; \
#		rm --force lab/src/*.aux lab/src/*.log lab/src/*.pdf; \

lab-bench: lab-bench-8-queues-1-deque lab-bench-8-queues-2-spsc lab-bench-8-queues-3-spsc-slice lab-bench-8-queues-5-reserving-mpmc lab-bench-8-queues-6-mpmc

lab-bench-8-queues-1-deque: install clean
	@( \
		cd ku; \
		cargo bench --no-run; \
		cargo bench --bench 8-queues-1-deque invariants/ 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-1-deque-bench.md; \
		cd - >/dev/null; \
		cp target/criterion/invariants/1/report/violin.svg lab/src/8-queues-1-invariants-time.svg.criterion; \
		cp target/criterion/invariants/report/lines_throughput.svg lab/src/8-queues-1-invariants-throughput.svg.criterion; \
		cd ku; \
		cargo bench --features benchmark --bench 8-queues-1-deque invariants/; \
		cd - >/dev/null; \
		cp target/criterion/invariants/1/report/violin.svg lab/src/8-queues-1-invariants-vs-other-implementations-time.svg.criterion; \
		cp target/criterion/invariants/report/lines_throughput.svg lab/src/8-queues-1-invariants-vs-other-implementations-throughput.svg.criterion; \
		for element_count in 1 1000; do \
			critcmp --filter "invariants/[0-9]\. .*/$$element_count$$" new | \
				sed 's@^invariants/[0-9]*\. @@g;s@<.*>@@g;s@^\([^/]*\)/\([0-9]*\) *[0-9.]* *\([0-9.±]* *[^ ]*\) *\(.*\)@\2 | \1 | \3 | \4@g;' | \
				tail --lines=+3 \
				> lab/src/8-queues-1-invariants-vs-other-implementations.$$element_count.md; \
			head --lines=4 lab/src/8-queues-1-invariants-vs-other-implementations.$$element_count.md > lab/src/8-queues-1-invariants.$$element_count.md; \
		done; \
		critcmp --filter 'invariants/.*(infinite_indices|sentinel)' new > lab/src/8-queues-1-invariants-critcmp.md; \
		for i in lab/src/*.svg.criterion; do \
			sed -i 's@\s*$$@@g;s@rgb(178,  34,  34)@rgb(128, 192, 255)@g;s@rgb( 46, 139,  87)@rgb(112, 192, 160)@g;s@rgb(  0, 139, 139)@rgb(224, 128, 144)@g;s@rgb(255, 215,   0)@rgb(144, 144, 255)@g;s@rgb(  0,   0, 139)@rgb(176, 128, 224)@g;s@rgb(220,  20,  60)@rgb(160, 160, 192)@g;s@rgb(139,   0, 139)@rgb( 96, 144, 172)@g;s@rgb(  0, 255, 127)@rgb(176, 160, 144)@g;' $$i; \
		done; \
	)

lab-bench-8-queues-2-spsc: install clean
	@( \
		cd ku; \
		cargo bench --no-run; \
		cargo bench --bench 8-queues-2-spsc sequential/ 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-2-sequential-bench.md; \
		cargo bench --bench 8-queues-2-spsc concurrent/ 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-2-concurrent-bench.md; \
		cargo bench --bench 8-queues-2-spsc bulk/ 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-2-bulk-bench.md; \
		cargo bench --bench 8-queues-2-spsc roundtrip/ 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-2-roundtrip-bench.md; \
		cargo bench --features benchmark --bench 8-queues-2-spsc sequential/; \
		cargo bench --features benchmark --bench 8-queues-2-spsc concurrent/; \
		cargo bench --features benchmark --bench 8-queues-2-spsc bulk/; \
		cargo bench --features benchmark --bench 8-queues-2-spsc roundtrip/; \
		cd - >/dev/null; \
		cp target/criterion/sequential/1000/report/violin.svg lab/src/8-queues-2-sequential-time.svg.criterion; \
		cp target/criterion/sequential/report/lines_throughput.svg lab/src/8-queues-2-sequential-throughput.svg.criterion; \
		cp target/criterion/concurrent/1000000/report/violin.svg lab/src/8-queues-2-concurrent-time.svg.criterion; \
		cp target/criterion/concurrent/report/lines_throughput.svg lab/src/8-queues-2-concurrent-throughput.svg.criterion; \
		cp target/criterion/bulk/1000000/report/violin.svg lab/src/8-queues-2-concurrent-time-bulk.svg.criterion; \
		cp target/criterion/bulk/report/lines_throughput.svg lab/src/8-queues-2-concurrent-throughput-bulk.svg.criterion; \
		cp 'target/criterion/roundtrip/spsc roundtrip_u32, 16384_/1/report/pdf.svg' lab/src/8-queues-2-roundtrip.svg.criterion; \
		critcmp --filter 'sequential/.*(spsc|sentinel)' new > lab/src/8-queues-2-sequential-critcmp.md; \
		critcmp --filter 'concurrent/1.*spsc' new > lab/src/8-queues-2-concurrent-critcmp.md; \
		critcmp --filter 'bulk/' new > lab/src/8-queues-2-bulk-critcmp.md; \
		for i in lab/src/*.svg.criterion; do \
			sed -i 's@\s*$$@@g;s@rgb(178,  34,  34)@rgb(128, 192, 255)@g;s@rgb( 46, 139,  87)@rgb(112, 192, 160)@g;s@rgb(  0, 139, 139)@rgb(224, 128, 144)@g;s@rgb(255, 215,   0)@rgb(144, 144, 255)@g;s@rgb(  0,   0, 139)@rgb(176, 128, 224)@g;s@rgb(220,  20,  60)@rgb(160, 160, 192)@g;s@rgb(139,   0, 139)@rgb( 96, 144, 172)@g;s@rgb(  0, 255, 127)@rgb(176, 160, 144)@g;' $$i; \
		done; \
	)

lab-bench-8-queues-3-spsc-slice: install clean
	@( \
		cd ku; \
		cargo bench --no-run; \
		cargo bench --bench 8-queues-3-spsc-slice 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-3-spsc-slice-bench.md; \
		cargo bench --features benchmark --bench 8-queues-3-spsc-slice; \
		cd - >/dev/null; \
		cp target/criterion/slice/report/lines_throughput.svg lab/src/8-queues-3-spsc-slice-throughput.svg.criterion; \
		critcmp --filter 'slice/.*(slice spsc|double)' new > lab/src/8-queues-3-spsc-slice-critcmp.md; \
		for i in lab/src/*.svg.criterion; do \
			sed -i 's@\s*$$@@g;s@rgb(178,  34,  34)@rgb(128, 192, 255)@g;s@rgb( 46, 139,  87)@rgb(112, 192, 160)@g;s@rgb(  0, 139, 139)@rgb(224, 128, 144)@g;s@rgb(255, 215,   0)@rgb(144, 144, 255)@g;s@rgb(  0,   0, 139)@rgb(176, 128, 224)@g;s@rgb(220,  20,  60)@rgb(160, 160, 192)@g;s@rgb(139,   0, 139)@rgb( 96, 144, 172)@g;s@rgb(  0, 255, 127)@rgb(176, 160, 144)@g;' $$i; \
		done; \
	)

lab-bench-8-queues-5-reserving-mpmc: install clean
	@( \
		cd ku; \
		cargo bench --no-run; \
		cargo bench --bench 8-queues-5-reserving-mpmc 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-5-reserving-mpmc-bench.md; \
		cargo bench --features benchmark --bench 8-queues-5-reserving-mpmc; \
		cd - >/dev/null; \
		cp target/criterion/reserving_mpmc/1000000/report/violin.svg lab/src/8-queues-5-reserving-mpmc-time.svg.criterion; \
		cp target/criterion/reserving_mpmc/report/lines_throughput.svg lab/src/8-queues-5-reserving-mpmc-throughput.svg.criterion; \
		cp 'target/criterion/roundtrip/reserving mpmc roundtrip_u32, 16384_/1/report/pdf.svg' lab/src/8-queues-5-reserving-mpmc-roundtrip.svg.criterion; \
		critcmp --filter 'reserving_mpmc/' new > lab/src/8-queues-5-reserving-mpmc-critcmp.md; \
		for i in lab/src/*.svg.criterion; do \
			sed -i 's@\s*$$@@g;s@rgb(178,  34,  34)@rgb(128, 192, 255)@g;s@rgb( 46, 139,  87)@rgb(112, 192, 160)@g;s@rgb(  0, 139, 139)@rgb(224, 128, 144)@g;s@rgb(255, 215,   0)@rgb(144, 144, 255)@g;s@rgb(  0,   0, 139)@rgb(176, 128, 224)@g;s@rgb(220,  20,  60)@rgb(160, 160, 192)@g;s@rgb(139,   0, 139)@rgb( 96, 144, 172)@g;s@rgb(  0, 255, 127)@rgb(176, 160, 144)@g;' $$i; \
		done; \
	)

lab-bench-8-queues-6-mpmc: install clean
	@( \
		cd ku; \
		cargo bench --no-run; \
		cargo bench --bench 8-queues-6-mpmc 2>&1 | \
			sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' | \
			tee ../lab/src/8-queues-6-mpmc-bench.md; \
		cargo bench --features benchmark --bench 8-queues-6-mpmc; \
		cd - >/dev/null; \
		cp target/criterion/mpmc/1000000/report/violin.svg lab/src/8-queues-6-mpmc-time.svg.criterion; \
		cp target/criterion/mpmc/report/lines_throughput.svg lab/src/8-queues-6-mpmc-throughput.svg.criterion; \
		cp 'target/criterion/roundtrip/mpmc roundtrip_u32, 16384_/1/report/pdf.svg' lab/src/8-queues-6-mpmc-roundtrip.svg.criterion; \
		critcmp --filter 'mpmc/' new > lab/src/8-queues-6-mpmc-critcmp.md; \
		for i in lab/src/*.svg.criterion; do \
			sed -i 's@\s*$$@@g;s@rgb(178,  34,  34)@rgb(128, 192, 255)@g;s@rgb( 46, 139,  87)@rgb(112, 192, 160)@g;s@rgb(  0, 139, 139)@rgb(224, 128, 144)@g;s@rgb(255, 215,   0)@rgb(144, 144, 255)@g;s@rgb(  0,   0, 139)@rgb(176, 128, 224)@g;s@rgb(220,  20,  60)@rgb(160, 160, 192)@g;s@rgb(139,   0, 139)@rgb( 96, 144, 172)@g;s@rgb(  0, 255, 127)@rgb(176, 160, 144)@g;' $$i; \
		done; \
	)

lab-test: install
	@( \
		cd ku; \
		cargo test --no-run; \
		for test in 8-queues-1-deque 8-queues-2-spsc 8-queues-3-spsc-slice 8-queues-4-sharded-mpmc 8-queues-5-reserving-mpmc 8-queues-6-mpmc; do \
			cargo test --test $$test 2>&1 | \
				sed 's@/home/[a-z-]*/tmp/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/[a-z-]*/@/.../nikka/@g;s@/home/[a-z-]*/\.@/.../.@g' \
				> ../lab/src/$$test-test.md || \
				exit 1; \
		done; \
		cd - >/dev/null; \
		rm --force lab/src/*.aux lab/src/*.log lab/src/*.pdf; \
	)

lectures: $(LECTURES_PDF)

lectures/%.pdf: lectures/%.md lectures/%.img/*
	pandoc --pdf-engine=xelatex -V mainfont="DejaVu Sans" -t beamer $< -o $@

fs-dump:
	@xxd fs.img | sed 's@.*0000 0000 0000 0000 0000 0000 0000 0000.*@...@' | uniq

#		for crate in $(CRATES) user/lib; do \

doc: FORCE
	@( \
		set -x; \
		rm --force --recursive doc; \
		mkdir doc; \
		for crate in ku text user/lib user/cow_fork user/eager_fork kernel; do \
			cd $$crate; \
			if [ .$$crate = .ku ]; then \
				feature_flags='--features allocator-statistics'; \
			else \
				feature_flags=''; \
			fi; \
			cargo doc --document-private-items $$feature_flags; \
			cd - >/dev/null; \
			rm -rf doc/$$(basename $$crate); \
			for src in target/doc/*/; do dst=$${src#target/}; if [ ! -e $$dst ]; then mv $$src $$dst; fi; done; \
			for src in target/kernel/doc/*/; do dst=$${src#target/kernel/}; if [ ! -e $$dst ]; then mv $$src $$dst; fi; done; \
			for i in $$(find doc/$$(basename $$crate) -name enum.\*.html -o -name struct.\*.html -o -name type.\*.html -o -name trait.\*.html -o -name fn.\*.html); do \
				sed 's@<html[^>]*>@\0\n@g;s@</html>@\n\0@g;s@<pre [^>]*>@\n\0@g;s@</pre>@</pre>\n@g;s@</details>@@g;s@<details class="toggle top-doc" open><summary class="hideme"><span>Expand description</span></summary>@@g;s@<details class="toggle type-contents-toggle"><summary class="hideme"><span>Show [^ ]* variants</span></summary>@@g;s@\(<a [^>]* *href="\)\([^#][^:">]*\)"@\1../../'$$(dirname $$i)'/\2"@g;s@\(<a [^>]* *href="\)#@\1../../'$$i'#@g;s@\<pub\(([^)]*)\)\? [ a-zC86&;-]*\(fn\|enum\|struct\|trait\)\>@\2@g;' $$i | \
					sed 's@\( href=[^ >]*["/]\)[^/.]*/\.\./@\1@g;' | \
					sed 's@\( href=[^ >]*["/]\)[^/.]*/\.\./@\1@g;' \
					> $$i.processed; \
			done; \
			for type in $$(find doc/$$(basename $$crate) -name enum.\*.html -o -name struct.\*.html -o -name type.\*.html); do \
				sed 's@<pre class="rust item-decl">@\n\0@g;s@<h2 id="implementations" class="section-header">@\n\0\n@g;s@<details@\n\0@g;s@</section>@\n\0@g;' $$type.processed | \
					sed '/<html/,/>Source<\/a>/d;/\(<h2 id="[a-zA-Z-]*\(implementations\|methods\)[a-zA-Z-]*" class="section-header">\|<\/section>\)/,/<\/html>/d;' | \
					sed 's@<span id="structfield\.[^"]*" class="structfield section-header">@<li>\0@g;' | \
					sed 's@<div class="sub-variant-field"><span id="variant\.[^.]*\.field.[^"]*" class="section-header">@<li>\0@g;' | \
					sed 's@<h2 id="fields" class="fields section-header">[^<]*Fields[^§]*§</a></h2>@<ul>\n@g;' | \
					sed 's@<h2 id="variants" class="variants section-header">Variants[^§]*§</a></h2>@<ul>\n@g;' | \
					sed 's@<h2 id="[^"]*"><a [^>]*>§</a>\([^<]*\)</h2>@<h5>\1</h5>@g;' | \
					sed 's@\(<div class="sub-variant" id="variant\.[^"]*">\)<h4>Fields</h4>@\1\n<ul>\n@g;' | \
					sed 's@<section id="variant\.[^"]*"[^>]*>\(<a href=[^>]*>\)§</a><h3 class="code-header">\(.*\)</h3></section>@<li><code>\1\2</a></code>@g;' | \
					sed 's@\(<a href="[^>]*>\)§</a><code>\([^:]*\):@<code>\1\2</a>:@g;' | \
					sed 's@</div><li>@</div></li>\n<li>@g;' | \
					sed '1i<blockquote>' | \
					sed '$$a</blockquote>' \
					> $$type.declaration; \
			done; \
			for type in $$(find doc/$$(basename $$crate) -name enum.\*.html -o -name struct.\*.html -o -name type.\*.html -o -name trait.\*.html); do \
				for method in $$(grep -o '<section id="\(ty\)\?method.[^"]*"' $$type | cut --fields=2 --delimiter=. | cut --fields=1 --delimiter='"'); do \
					sed 's@</div>@\n\0\n@g;s@class="method trait-impl">@\0\n@g;s@class="method">@\0\n@g;' $$type.processed | \
					sed '/<html/,/<section id="\(ty\)\?method.'$$method'"/d;/<\/div>/,/<\/html>/d;' | \
					sed 's@<a [^>]*class="fn"[^>]*>@\0'$$(basename $$type | cut --fields=2 --delimiter=.)'::@g;s@<a [^>]*>\(Source\|§\)</a>@@g;s@<h4 class="code-header">@<pre><code>@g;s@</h4>@</code></pre>@g;s@<h5 id="[^"]*">@<h5>@g;' | \
					sed '1i<blockquote>' | \
					sed '$$a</div></blockquote>' \
					> $$type.method.$$method; \
				done; \
			done; \
			for fn in $$(find doc/$$(basename $$crate) -name fn.\*.html); do \
				sed 's@</div>@\0\n@g;s@<code>.*\<fn @\0'$$(dirname $$fn | sed 's@^doc/@@g;s@/@::@g;')'::@g;' $$fn.processed | \
					sed '/<html/,/>Source<\/a>/d;/<\/div>/,/<\/html>/d;' | \
					sed 's@<a [^>]*>§</a>@@g;s@<h2@<h5@g;s@</h2>@</h5>@g;' | \
					sed '1i<blockquote>' | \
					sed '$$a</blockquote>' \
					> $$fn.fn; \
			done; \
		done; \
		cargo doc --document-private-items; \
		for src in target/doc/*/; do dst=$${src#target/}; if [ ! -e $$dst ]; then mv $$src $$dst; fi; done; \
		rm --force $$(grep --files-with-matches 'compose::begin_private' $$(find doc/src -name \*.rs.html) ); \
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
		lab/src/*-lab-dependencies.dot \
		lectures/*.pdf \
		$(find . -name \*.rs.bk)

stat:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --stat

compose:
	@cargo run --manifest-path tools/compose/Cargo.toml -- --in-path . --out-path .

install:
	@(which rustup > /dev/null || (curl https://sh.rustup.rs -sSf | sh))
	@(which bootimage > /dev/null || (cargo install --version 0.10.3 bootimage))
	@(which rustfilt > /dev/null || (cargo install rustfilt))
	@(which cargo-expand > /dev/null || (cargo install cargo-expand))
	@(which cargo-spellcheck  > /dev/null || (cargo install cargo-spellcheck))
	@(which critcmp > /dev/null || (cargo install critcmp))
	@(which typos > /dev/null || (cargo install typos-cli))
	@(which taplo > /dev/null || (cargo install taplo-cli))

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

pdf: FORCE
	rm -rf pdf
	mkdir pdf
	for i in $$(jj file list | grep '\.rs$$' | sort --unique); do \
		j=$$(echo $$i | sed 's@^./@@;s@/@,@g'); \
		trueprint \
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
		$$i | \
		ps2pdf /dev/stdin pdf/$$j.pdf; \
	done

nikka.enscript.pdf: FORCE
	enscript -1rE -C -H -j -O --toc -b '$n|20$D $C|$% / $= ; $p' -T 2 -f 'Courier@12' -M A5 $$(find . -name '*.rs' | sort --unique) -o nikka.enscript.pdf

src.pdf: doc FORCE
	rm -rf doc/src.pdf
	cp -pr doc/src doc/src.pdf
	cargo doc --document-private-items --no-default-features
	rm --force --recursive doc
	mv target/doc .
	for i in $$(find doc/src.pdf '*.html'); do \
		echo "-------------------------------------- "$i \
		sed -i '/^<span id="[0-9]*">[0-9]*<\/span>/d;s@<body class="rustdoc source">.*@<body>@' $$i; \
		j=$$(echo $$i | sed 's@\.html@.pdf@'); \
		wkhtmltopdf \
			--enable-local-file-access \
			--minimum-font-size 32 \
			--orientation Landscape \
			--page-size A5 \
			$$i $$j; \
		rm -f $$i; \
	done
	mv doc/src.pdf .
