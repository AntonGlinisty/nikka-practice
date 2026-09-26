set architecture i386:x86-64
set disassembly-flavor intel
set history filename .gdb_history
set history save on
set print asm-demangle on
set print pretty on
target remote localhost:1234
break kernel::kernel_main
