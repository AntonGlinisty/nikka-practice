## Изменение размера аллокатора фреймов

### Задача 8 --- изменение размера аллокатора фреймов

#### Инициализация описателей фреймов

Реализуйте метод
[`FrameAllocator::resize()`](../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html#method.resize)
и функцию
[`frame_allocator::resize()`](../../doc/kernel/memory/frame_allocator/fn.resize.html)
в файле
[`kernel/src/memory/frame_allocator.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/src/memory/frame_allocator.rs).

{{#include ../../doc/kernel/memory/frame_allocator/struct.FrameAllocator.html.method.resize}}

{{#include ../../doc/kernel/memory/frame_allocator/fn.resize.html.fn}}


### Проверьте себя

Запустите тест:

```console
$ (cd kernel; cargo test --test 2-mm-8-resize-frame-allocator)
...
running 12 tests

2_mm_8_resize_frame_allocator::all_frames_are_freed_on_drop-
19:22:59 0 I duplicate; address_space = "process" @ 0p7A_D000
19:22:59 0 D page = 4294967296 @ 0v1000_0000_0000; frame = 1340 @ 0p53_C000; flags = 0x8000000000000005
19:22:59 0 D page = 4429185024 @ 0v1080_0000_0000; frame = 1336 @ 0p53_8000; flags = 0x8000000000000007
19:22:59 0 D page = 4563402752 @ 0v1100_0000_0000; frame = 1332 @ 0p53_4000; flags = 0x5
19:22:59 0 I drop; address_space = "process" @ 0p7A_D000
2_mm_8_resize_frame_allocator::all_frames_are_freed_on_drop [passed]

2_mm_8_resize_frame_allocator::allocated_frames_are_not_used
19:23:00.731 0 D start_free_frames = 30809; frame_count = 32768
19:23:00.741 0 D poisoning the free memory; frames_poisoned = 0; frame = 1293 @ 0p50_D000; poison = 16045690984833335023
19:23:01.377 0 D poisoning the free memory; frames_poisoned = 1000; frame = 31737 @ 0p7BF_9000; poison = 16045690984833335023
19:23:02.011 0 D poisoning the free memory; frames_poisoned = 2000; frame = 30737 @ 0p781_1000; poison = 16045690984833335023
19:23:02.649 0 D poisoning the free memory; frames_poisoned = 3000; frame = 29737 @ 0p742_9000; poison = 16045690984833335023
19:23:03.285 0 D poisoning the free memory; frames_poisoned = 4000; frame = 28737 @ 0p704_1000; poison = 16045690984833335023
19:23:03.941 0 D poisoning the free memory; frames_poisoned = 5000; frame = 27737 @ 0p6C5_9000; poison = 16045690984833335023
19:23:04.577 0 D poisoning the free memory; frames_poisoned = 6000; frame = 26737 @ 0p687_1000; poison = 16045690984833335023
19:23:05.211 0 D poisoning the free memory; frames_poisoned = 7000; frame = 25737 @ 0p648_9000; poison = 16045690984833335023
19:23:05.847 0 D poisoning the free memory; frames_poisoned = 8000; frame = 24737 @ 0p60A_1000; poison = 16045690984833335023
19:23:06.483 0 D poisoning the free memory; frames_poisoned = 9000; frame = 23737 @ 0p5CB_9000; poison = 16045690984833335023
19:23:07.117 0 D poisoning the free memory; frames_poisoned = 10000; frame = 22737 @ 0p58D_1000; poison = 16045690984833335023
19:23:07.757 0 D poisoning the free memory; frames_poisoned = 11000; frame = 21737 @ 0p54E_9000; poison = 16045690984833335023
19:23:08.393 0 D poisoning the free memory; frames_poisoned = 12000; frame = 20737 @ 0p510_1000; poison = 16045690984833335023
19:23:09.027 0 D poisoning the free memory; frames_poisoned = 13000; frame = 19737 @ 0p4D1_9000; poison = 16045690984833335023
19:23:09.661 0 D poisoning the free memory; frames_poisoned = 14000; frame = 18737 @ 0p493_1000; poison = 16045690984833335023
19:23:10.299 0 D poisoning the free memory; frames_poisoned = 15000; frame = 17737 @ 0p454_9000; poison = 16045690984833335023
19:23:10.933 0 D poisoning the free memory; frames_poisoned = 16000; frame = 16737 @ 0p416_1000; poison = 16045690984833335023
19:23:11.571 0 D poisoning the free memory; frames_poisoned = 17000; frame = 15605 @ 0p3CF_5000; poison = 16045690984833335023
19:23:12.207 0 D poisoning the free memory; frames_poisoned = 18000; frame = 14605 @ 0p390_D000; poison = 16045690984833335023
19:23:12.845 0 D poisoning the free memory; frames_poisoned = 19000; frame = 13605 @ 0p352_5000; poison = 16045690984833335023
19:23:13.485 0 D poisoning the free memory; frames_poisoned = 20000; frame = 12605 @ 0p313_D000; poison = 16045690984833335023
19:23:14.121 0 D poisoning the free memory; frames_poisoned = 21000; frame = 11605 @ 0p2D5_5000; poison = 16045690984833335023
19:23:14.757 0 D poisoning the free memory; frames_poisoned = 22000; frame = 10605 @ 0p296_D000; poison = 16045690984833335023
19:23:15.395 0 D poisoning the free memory; frames_poisoned = 23000; frame = 9605 @ 0p258_5000; poison = 16045690984833335023
19:23:16.031 0 D poisoning the free memory; frames_poisoned = 24000; frame = 8605 @ 0p219_D000; poison = 16045690984833335023
19:23:16.665 0 D poisoning the free memory; frames_poisoned = 25000; frame = 7605 @ 0p1DB_5000; poison = 16045690984833335023
19:23:17.301 0 D poisoning the free memory; frames_poisoned = 26000; frame = 6605 @ 0p19C_D000; poison = 16045690984833335023
19:23:17.937 0 D poisoning the free memory; frames_poisoned = 27000; frame = 5605 @ 0p15E_5000; poison = 16045690984833335023
19:23:18.577 0 D poisoning the free memory; frames_poisoned = 28000; frame = 4605 @ 0p11F_D000; poison = 16045690984833335023
19:23:19.213 0 D poisoning the free memory; frames_poisoned = 29000; frame = 3605 @ 0pE1_5000; poison = 16045690984833335023
19:23:19.849 0 D poisoning the free memory; frames_poisoned = 30000; frame = 2605 @ 0pA2_D000; poison = 16045690984833335023
19:23:20.369 0 D start_free_frames = 30809; end_free_frames = 0; frames_poisoned = 30809
19:23:21.171 0 D free_frames = 30809
2_mm_8_resize_frame_allocator::allocated_frames_are_not_used [passed]

2_mm_8_resize_frame_allocator::allocated_frames_are_unique--
19:23:21.257 0 D free_frames = 30809; frame_count = 32768
19:23:21.355 0 D free_frames = 0
19:23:21.865 0 D free_frames = 30809
2_mm_8_resize_frame_allocator::allocated_frames_are_unique [passed]

2_mm_8_resize_frame_allocator::basic_frame_allocator_functions
19:23:21.909 0 D frames = [Frame(1965 @ 0p7A_D000), Frame(1329 @ 0p53_1000)]; frame_count = 32768
19:23:21.925 0 D reallocate_last_freed_frame = Frame(1965 @ 0p7A_D000)
2_mm_8_resize_frame_allocator::basic_frame_allocator_functions [passed]

2_mm_8_resize_frame_allocator::duplicate_and_drop_mapping---
19:23:21.955 0 D one address space; virt = 0vF7F_FFF4_0000; frame = 1965 @ 0p7A_D000; reference_count = 1
19:23:22.275 0 I duplicate; address_space = "process" @ 0p53_1000
19:23:22.283 0 D two identical address spaces; virt = 0vF7F_FFF4_0000; frame = 1965 @ 0p7A_D000; reference_count = 2
19:23:22.299 0 D ptes = [1965 @ 0p7A_D000 -----DA---W-P(0x8000000000000063), 1965 @ 0p7A_D000 -----DA---W-P(0x8000000000000063)]
19:23:22.315 0 D pte_addresses = [Virt(0vFFFF800003FFBA00), Virt(0vFFFF80000053EA00)]
19:23:22.327 0 I drop; address_space = "process" @ 0p53_1000
19:23:22.905 0 D back to one address space; virt = 0vF7F_FFF4_0000; frame = 1965 @ 0p7A_D000; reference_count = 1
2_mm_8_resize_frame_allocator::duplicate_and_drop_mapping [passed]

2_mm_8_resize_frame_allocator::duplicate_only_kernel--------
19:23:22.937 0 D page = Page(34359738305 @ 0v7FFF_FFFC_1000)
19:23:23.261 0 I duplicate; address_space = "process" @ 0p53_1000
19:23:23.269 0 D ptes = [1965 @ 0p7A_D000 ---------U--P(0x8000000000000005), <non-present>]
19:23:23.283 0 I drop; address_space = "process" @ 0p53_1000
2_mm_8_resize_frame_allocator::duplicate_only_kernel [passed]

2_mm_8_resize_frame_allocator::duplicate_works--------------
19:23:23.887 0 D original address space initializes the variable; variable = 0
19:23:24.203 0 I duplicate; address_space = "process" @ 0p53_1000
19:23:24.211 0 I switch to; address_space = "process" @ 0p53_1000
19:23:24.219 0 D inside the duplicate address space
19:23:24.227 0 D duplicate address space modifies the variable; variable = 1
19:23:24.233 0 I switch to; address_space = "base" @ 0p1000
19:23:24.241 0 I drop; address_space = "process" @ 0p53_1000
19:23:24.751 0 D original address space reads the variable; variable = 1
2_mm_8_resize_frame_allocator::duplicate_works----- [passed]

2_mm_8_resize_frame_allocator::garbage_after_unmap----------
19:23:24.797 0 D pte = <non-present>; expected_pte = <non-present>
2_mm_8_resize_frame_allocator::garbage_after_unmap- [passed]

2_mm_8_resize_frame_allocator::garbage_in_duplicate---------
19:23:25.309 0 I duplicate; address_space = "process" @ 0p54_0000
19:23:25.319 0 D ptes = [1965 @ 0p7A_D000 ----------W-P(0x8000000000000003), 1965 @ 0p7A_D000 ----------W-P(0x8000000000000003)]
19:23:25.333 0 D pte_addresses = [Virt(0vFFFF80000053F000), Virt(0vFFFF800000535000)]
19:23:25.565 0 I drop; address_space = "process" @ 0p54_0000
2_mm_8_resize_frame_allocator::garbage_in_duplicate [passed]

2_mm_8_resize_frame_allocator::sanity_check-----------------
19:23:26.241 0 D free_frames = 30864; min_free_frames = 28672; qemu_memory_frames = 32768
2_mm_8_resize_frame_allocator::sanity_check-------- [passed]

2_mm_8_resize_frame_allocator::shared_memory----------------
19:23:26.271 0 D frame = 1965 @ 0p7A_D000
19:23:26.279 0 D pages = [Page(4160749373 @ 0vF7F_FFF3_D000), Page(4160749374 @ 0vF7F_FFF3_E000)]
19:23:26.295 0 D write_ptr = 0xf7ffff3e000; read_ptr = 0xf7ffff3d000
19:23:26.305 0 D write_value = 0; read_value = 0
19:23:26.311 0 D write_value = 1; read_value = 1
19:23:26.317 0 D write_value = 2; read_value = 2
2_mm_8_resize_frame_allocator::shared_memory------- [passed]

2_mm_8_resize_frame_allocator::tlb_flush--------------------
19:23:26.345 0 D frame = 1965 @ 0p7A_D000; page = 4160749372 @ 0vF7F_FFF3_C000; frame_mark = 1111111
19:23:26.359 0 D frame = 1344 @ 0p54_0000; page = 4160749371 @ 0vF7F_FFF3_B000; frame_mark = 2222222
19:23:26.373 0 D the page is currently mapped to the old frame and should contain its old mark; page = 4160749370 @ 0vF7F_FFF3_A000; old_frame = 1965 @ 0p7A_D000; current_mark = 1111111; old_mark = 1111111
19:23:26.395 0 D the page is currently mapped to the new frame and should contain its new mark; page = 4160749370 @ 0vF7F_FFF3_A000; new_frame = 1344 @ 0p54_0000; current_mark = 2222222; new_mark = 2222222
2_mm_8_resize_frame_allocator::tlb_flush----------- [passed]
19:23:26.421 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```


### Ориентировочный объём работ этой части лабораторки

```console
kernel/src/memory/frame_allocator.rs | 43 +++++++++++++++++++++++++++++++++++++++----
1 file changed, 39 insertions(+), 4 deletions(-)
```
