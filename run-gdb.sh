#!/usr/bin/env bash

test=$1

if [ -z "$test" ]; then
    echo "Usage:"
    echo "$0 [test-name]"
    echo "Eg.: $0 3-smp-5-ap-init"
    exit 1
fi

command=$(
    (
        cd kernel || true; \
        cargo test --no-run --test $test >/dev/null; \
        timeout 1 cargo test --test $test 2>/dev/null | grep qemu | head --lines 1 | cut --fields=2 --delimiter='`'
    )
)

echo "Waiting for GDB to attach. Run 'make gdb' in a separate console."

$command -gdb tcp::1234 -S
