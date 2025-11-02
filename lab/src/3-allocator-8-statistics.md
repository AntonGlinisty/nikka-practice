## Статистики аллокатора

Если вы поддержали
[сбор статистик аллокатора](../../lab/book/3-allocator-1-big.html#%D0%A1%D1%82%D0%B0%D1%82%D0%B8%D1%81%D1%82%D0%B8%D0%BA%D0%B8-%D0%B0%D0%BB%D0%BB%D0%BE%D0%BA%D0%B0%D1%82%D0%BE%D1%80%D0%B0-kuallocatorinfoinfo)
в предыдущих задачах,
запустите тест `3-allocator-1-big` из файла
[`kernel/tests/3-allocator-1-big.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/3-allocator-1-big.rs)
и тест `3-allocator-5-small` из файла
[`kernel/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/kernel/tests/3-allocator-5-small.rs).
А также стресс--тест `3-allocator-5-small` из файла
[`ku/tests/3-allocator-5-small.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-5-small.rs).
При этом укажите флаг `--features allocator-statistics`:

```console
$ (cd kernel && cargo test --features allocator-statistics --test 3-allocator-1-big)
...
running 10 tests

3_allocator_1_big::alignment--------------------------------
20:22:06 0 D align = 1
20:22:06 0 D align = 2
20:22:06 0 D align = 4
20:22:06 0 D align = 8
20:22:06 0 D align = 16
20:22:06 0 D align = 32
20:22:06 0 D align = 64
20:22:06 0 D align = 128
20:22:06 0 D align = 256
20:22:06 0 D align = 512
20:22:06 0 D align = 1024
20:22:06 0 D align = 2048
20:22:06 0 D align = 4096
3_allocator_1_big::alignment----------------------- [passed]

3_allocator_1_big::basic------------------------------------
20:22:06 0 D start_info = { allocations: 2340 - 2340 = 0, requested: 8.110 MiB - 8.110 MiB = 0 B, allocated: 9.488 MiB - 9.488 MiB = 0 B, pages: 1847 - 1684 = 163, loss: 652.000 KiB = 100.000% }
20:22:06 0 D box_contents = 2
20:22:06 0 D info = { allocations: 2341 - 2340 = 1, requested: 8.110 MiB - 8.110 MiB = 8 B, allocated: 9.488 MiB - 9.488 MiB = 8 B, pages: 1849 - 1684 = 165, loss: 659.992 KiB = 99.999% }
20:22:06 0 D info_diff = { allocations: 1 - 0 = 1, requested: 8 B - 0 B = 8 B, allocated: 8 B - 0 B = 8 B, pages: 2 - 0 = 2, loss: 7.992 KiB = 99.902% }
20:22:06 0 D end_info = { allocations: 2341 - 2341 = 0, requested: 8.110 MiB - 8.110 MiB = 0 B, allocated: 9.488 MiB - 9.488 MiB = 0 B, pages: 1849 - 1684 = 165, loss: 660.000 KiB = 100.000% }
20:22:06 0 D end_info_diff = { allocations: 1 - 1 = 0, requested: 8 B - 8 B = 0 B, allocated: 8 B - 8 B = 0 B, pages: 2 - 0 = 2, loss: 8.000 KiB = 100.000% }
3_allocator_1_big::basic--------------------------- [passed]

3_allocator_1_big::copy_mapping_corner_cases----------------
20:22:06 0 D different intersecting blocks of the same size, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_8000, 0v1000_0000_D000), size 20.000 KiB; flags = None
20:22:06 0 D different intersecting blocks of the same size, allowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_8000, 0v1000_0000_D000), size 20.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D different intersecting blocks of the same size, disallowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_8000, 0v1000_0000_D000), size 20.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D block size differs, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = None
20:22:06 0 D block size differs, allowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D block size differs, disallowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D old block is a strict superblock, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = None
20:22:06 0 D old block is a strict superblock, allowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D old block is a strict superblock, disallowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D block size differs, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = None
20:22:06 0 D block size differs, allowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D block size differs, disallowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D new block is a strict superblock, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = None
20:22:06 0 D new block is a strict superblock, allowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D new block is a strict superblock, disallowed flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; new_block = [0v1000_0000_5000, 0v1000_0000_A000), size 20.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D the same block, no flags; expected_result = Err(InvalidArgument); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = None
20:22:06 0 D the same block, allowed flags; expected_result = Ok(()); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D the same block, disallowed flags; expected_result = Err(PermissionDenied); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; flags = Some(0x8000000000000003)
20:22:06 0 D disjoint blocks of the same size, no flags; expected_result = Ok(()); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = None
20:22:06 0 D disjoint blocks of the same size, allowed flags; expected_result = Ok(()); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = Some(0x8000000000000005)
20:22:06 0 D disjoint blocks of the same size, disallowed flags; expected_result = Err(PermissionDenied); old_block = [0v1000_0000_0000, 0v1000_0000_1000), size 4.000 KiB; new_block = [0v1000_0000_6000, 0v1000_0000_7000), size 4.000 KiB; flags = Some(0x8000000000000003)
3_allocator_1_big::copy_mapping_corner_cases------- [passed]

3_allocator_1_big::copy_mapping_frame_references------------
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; translate_result = Err(NoPage)
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; translate_result = Err(NoPage)
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; translate_result = Err(NoPage)
20:22:06 0 D block = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738361 @ 0v7FFF_FFFF_9000; translate_result = Err(NoPage)
20:22:06 0 D page = 34359738362 @ 0v7FFF_FFFF_A000; translate_result = Err(NoPage)
20:22:06 0 D page = 34359738363 @ 0v7FFF_FFFF_B000; translate_result = Err(NoPage)
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; should_be_mapped = true
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007)
20:22:06 0 D block = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738361 @ 0v7FFF_FFFF_9000; pte = <non-present>
20:22:06 0 D page = 34359738362 @ 0v7FFF_FFFF_A000; pte = <non-present>
20:22:06 0 D page = 34359738363 @ 0v7FFF_FFFF_B000; pte = <non-present>
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; expected_reference_count = 1
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; should_be_mapped = true
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007)
20:22:06 0 D blocks should be equal; a = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; b = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB
20:22:06 0 D a_pte = Ok(32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007)); b_pte = Ok(32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007))
20:22:06 0 D a_pte = Ok(32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007)); b_pte = Ok(32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007))
20:22:06 0 D a_pte = Ok(32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007)); b_pte = Ok(32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007))
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; expected_reference_count = 2
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007); reference_count = 2
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007); reference_count = 2
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007); reference_count = 2
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = <non-present>
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = <non-present>
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = <non-present>
20:22:06 0 D block = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB; should_be_mapped = true
20:22:06 0 D page = 34359738361 @ 0v7FFF_FFFF_9000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738362 @ 0v7FFF_FFFF_A000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007)
20:22:06 0 D page = 34359738363 @ 0v7FFF_FFFF_B000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007)
20:22:06 0 D block = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB; expected_reference_count = 1
20:22:06 0 D page = 34359738361 @ 0v7FFF_FFFF_9000; pte = 32593 @ 0p7F5_1000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D page = 34359738362 @ 0v7FFF_FFFF_A000; pte = 32584 @ 0p7F4_8000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D page = 34359738363 @ 0v7FFF_FFFF_B000; pte = 32583 @ 0p7F4_7000 ---------UW-P(0x8000000000000007); reference_count = 1
20:22:06 0 D block = [0v7FFF_FFFF_C000, 0v7FFF_FFFF_F000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738364 @ 0v7FFF_FFFF_C000; pte = <non-present>
20:22:06 0 D page = 34359738365 @ 0v7FFF_FFFF_D000; pte = <non-present>
20:22:06 0 D page = 34359738366 @ 0v7FFF_FFFF_E000; pte = <non-present>
20:22:06 0 D block = [0v7FFF_FFFF_9000, 0v7FFF_FFFF_C000), size 12.000 KiB; should_be_mapped = false
20:22:06 0 D page = 34359738361 @ 0v7FFF_FFFF_9000; pte = <non-present>
20:22:06 0 D page = 34359738362 @ 0v7FFF_FFFF_A000; pte = <non-present>
20:22:06 0 D page = 34359738363 @ 0v7FFF_FFFF_B000; pte = <non-present>
3_allocator_1_big::copy_mapping_frame_references--- [passed]

3_allocator_1_big::copy_mapping_same_block------------------
3_allocator_1_big::copy_mapping_same_block--------- [passed]

3_allocator_1_big::grow_and_shrink--------------------------
20:22:06 0 D start_info = { allocations: 2341 - 2341 = 0, requested: 8.110 MiB - 8.110 MiB = 0 B, allocated: 9.488 MiB - 9.488 MiB = 0 B, pages: 1849 - 1684 = 165, loss: 660.000 KiB = 100.000% }
20:22:06 0 D contents_sum = 75491328; push_sum = 75491328
20:22:06 0 D info = { allocations: 2354 - 2353 = 1, requested: 8.360 MiB - 8.235 MiB = 128.000 KiB, allocated: 9.738 MiB - 9.613 MiB = 128.000 KiB, pages: 1914 - 1715 = 199, loss: 668.000 KiB = 83.920% }
20:22:06 0 D info_diff = { allocations: 13 - 12 = 1, requested: 255.969 KiB - 127.969 KiB = 128.000 KiB, allocated: 255.969 KiB - 127.969 KiB = 128.000 KiB, pages: 65 - 31 = 34, loss: 8.000 KiB = 5.882% }
20:22:06 0 D contents_sum = 75491328; pop_sum = 75491328
20:22:06 0 D end_info = { allocations: 2368 - 2368 = 0, requested: 8.485 MiB - 8.485 MiB = 0 B, allocated: 9.863 MiB - 9.863 MiB = 0 B, pages: 1947 - 1778 = 169, loss: 676.000 KiB = 100.000% }
20:22:06 0 D end_info_diff = { allocations: 27 - 27 = 0, requested: 383.961 KiB - 383.961 KiB = 0 B, allocated: 383.961 KiB - 383.961 KiB = 0 B, pages: 98 - 94 = 4, loss: 16.000 KiB = 100.000% }
3_allocator_1_big::grow_and_shrink----------------- [passed]

3_allocator_1_big::grow_and_shrink_unmap_old_block----------
20:22:06 0 D initialize = Garbage
20:22:06 0 D small_block = [0v7FFF_FFFF_2000, 0v7FFF_FFFF_6000), size 16.000 KiB; big_block = [0v7FFF_FFFE_A000, 0v7FFF_FFFF_2000), size 32.000 KiB
20:22:06 0 D unmapped_pages = 4
20:22:06 0 D big_block = [0v7FFF_FFFE_A000, 0v7FFF_FFFF_2000), size 32.000 KiB; medium_block = [0v7FFF_FFFE_A000, 0v7FFF_FFFF_0000), size 24.000 KiB
20:22:06 0 D unmapped_pages = 2
20:22:06 0 D initialize = Zero
20:22:06 0 D small_block = [0v7FFF_FFFE_6000, 0v7FFF_FFFE_A000), size 16.000 KiB; big_block = [0v7FFF_FFFD_E000, 0v7FFF_FFFE_6000), size 32.000 KiB
20:22:06 0 D unmapped_pages = 4
20:22:06 0 D big_block = [0v7FFF_FFFD_E000, 0v7FFF_FFFE_6000), size 32.000 KiB; medium_block = [0v7FFF_FFFD_E000, 0v7FFF_FFFE_4000), size 24.000 KiB
20:22:06 0 D unmapped_pages = 2
3_allocator_1_big::grow_and_shrink_unmap_old_block- [passed]

3_allocator_1_big::paged_realloc_is_cheap-------------------
20:22:06 0 D block = [0vF7F_FB23_2000, 0vF7F_FB23_3000), size 4.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB23_0000, 0vF7F_FB23_2000), size 8.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB22_C000, 0vF7F_FB23_0000), size 16.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB22_4000, 0vF7F_FB22_C000), size 32.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB22_4000), size 64.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003), 32569 @ 0p7F3_9000 ----------W-P(0x8000000000000003), 32570 @ 0p7F3_A000 ----------W-P(0x8000000000000003), 32571 @ 0p7F3_B000 ----------W-P(0x8000000000000003), 32548 @ 0p7F2_4000 ----------W-P(0x8000000000000003), 32549 @ 0p7F2_5000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB22_3000), size 60.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003), 32569 @ 0p7F3_9000 ----------W-P(0x8000000000000003), 32570 @ 0p7F3_A000 ----------W-P(0x8000000000000003), 32571 @ 0p7F3_B000 ----------W-P(0x8000000000000003), 32548 @ 0p7F2_4000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB22_2000), size 56.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003), 32569 @ 0p7F3_9000 ----------W-P(0x8000000000000003), 32570 @ 0p7F3_A000 ----------W-P(0x8000000000000003), 32571 @ 0p7F3_B000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB22_1000), size 52.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003), 32569 @ 0p7F3_9000 ----------W-P(0x8000000000000003), 32570 @ 0p7F3_A000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB22_0000), size 48.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003), 32569 @ 0p7F3_9000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_F000), size 44.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003), 32568 @ 0p7F3_8000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_E000), size 40.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003), 32567 @ 0p7F3_7000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_D000), size 36.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003), 32566 @ 0p7F3_6000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_C000), size 32.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003), 32565 @ 0p7F3_5000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_B000), size 28.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003), 32564 @ 0p7F3_4000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_A000), size 24.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003), 32575 @ 0p7F3_F000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_9000), size 20.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003), 32574 @ 0p7F3_E000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_8000), size 16.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003), 32573 @ 0p7F3_D000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_7000), size 12.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003), 32572 @ 0p7F3_C000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_6000), size 8.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003), 32577 @ 0p7F4_1000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0vF7F_FB21_4000, 0vF7F_FB21_5000), size 4.000 KiB; frames = [32576 @ 0p7F4_0000 ----------W-P(0x8000000000000003)]
20:22:06 0 D block = [0v1000, 0v1000), size 0 B; frames = []
3_allocator_1_big::paged_realloc_is_cheap---------- [passed]

3_allocator_1_big::shrink_is_not_a_noop---------------------
20:22:06 0 D initialize = Garbage
20:22:06 0 D old_block = [0v7FFF_FFFD_A000, 0v7FFF_FFFD_E000), size 16.000 KiB; new_block = [0v7FFF_FFFD_A000, 0v7FFF_FFFD_D000), size 12.000 KiB; no_remap_on_shrink = true
20:22:06 0 D initialize = Zero
20:22:06 0 D old_block = [0v7FFF_FFFD_6000, 0v7FFF_FFFD_A000), size 16.000 KiB; new_block = [0v7FFF_FFFD_6000, 0v7FFF_FFFD_9000), size 12.000 KiB; no_remap_on_shrink = true
3_allocator_1_big::shrink_is_not_a_noop------------ [passed]

3_allocator_1_big::stress-----------------------------------
20:22:06 0 D start_info = { allocations: 2408 - 2408 = 0, requested: 9.076 MiB - 9.076 MiB = 0 B, allocated: 10.454 MiB - 10.454 MiB = 0 B, pages: 2110 - 1929 = 181, loss: 724.000 KiB = 100.000% }
20:22:06 0 D vector_length = 500; fragmentation_loss = 0 B; max_fragmentation_loss = 16.000 MiB
20:22:06 0 D vector_length = 1000; fragmentation_loss = 0 B; max_fragmentation_loss = 16.000 MiB
20:22:06 0 D vector_length = 1500; fragmentation_loss = 0 B; max_fragmentation_loss = 16.000 MiB
20:22:06 0 D vector_length = 2000; fragmentation_loss = 0 B; max_fragmentation_loss = 16.000 MiB
20:22:06 0 D vector_length = 2500; fragmentation_loss = 0 B; max_fragmentation_loss = 19.531 MiB
20:22:06 0 D vector_length = 3000; fragmentation_loss = 0 B; max_fragmentation_loss = 23.438 MiB
20:22:06 0 D vector_length = 3500; fragmentation_loss = 0 B; max_fragmentation_loss = 27.344 MiB
20:22:06 0 D vector_length = 4000; fragmentation_loss = 0 B; max_fragmentation_loss = 31.250 MiB
20:22:06 0 D vector_length = 4500; fragmentation_loss = 0 B; max_fragmentation_loss = 35.156 MiB
20:22:06 0 D vector_length = 5000; fragmentation_loss = 0 B; max_fragmentation_loss = 39.062 MiB
20:22:06 0 D vector_length = 5500; fragmentation_loss = 0 B; max_fragmentation_loss = 42.969 MiB
20:22:06 0 D vector_length = 6000; fragmentation_loss = 0 B; max_fragmentation_loss = 46.875 MiB
20:22:06 0 D vector_length = 6500; fragmentation_loss = 0 B; max_fragmentation_loss = 50.781 MiB
20:22:06 0 D vector_length = 7000; fragmentation_loss = 0 B; max_fragmentation_loss = 54.688 MiB
20:22:06 0 D vector_length = 7500; fragmentation_loss = 0 B; max_fragmentation_loss = 58.594 MiB
20:22:06 0 D vector_length = 8000; fragmentation_loss = 0 B; max_fragmentation_loss = 62.500 MiB
20:22:06 0 D vector_length = 8500; fragmentation_loss = 0 B; max_fragmentation_loss = 66.406 MiB
20:22:06 0 D vector_length = 9000; fragmentation_loss = 0 B; max_fragmentation_loss = 70.312 MiB
20:22:06 0 D vector_length = 9500; fragmentation_loss = 0 B; max_fragmentation_loss = 74.219 MiB
20:22:06 0 D vector_length = 10000; fragmentation_loss = 0 B; max_fragmentation_loss = 78.125 MiB
20:22:07 0 D contents_sum = 333283335000; push_sum = 333283335000
20:22:07 0 D info = { allocations: 12425 - 2420 = 10005, requested: 9.403 MiB - 9.201 MiB = 206.969 KiB, allocated: 10.782 MiB - 10.579 MiB = 206.969 KiB, pages: 2192 - 1960 = 232, loss: 721.031 KiB = 77.697% }
20:22:07 0 D info_diff = { allocations: 10017 - 12 = 10005, requested: 334.938 KiB - 127.969 KiB = 206.969 KiB, allocated: 334.938 KiB - 127.969 KiB = 206.969 KiB, pages: 82 - 31 = 51, loss: 0 B = 0.000% }
20:22:07 0 D vector_length = 9500; fragmentation_loss = 960 B; max_fragmentation_loss = 74.219 MiB
20:22:07 0 D vector_length = 9000; fragmentation_loss = 4.844 KiB; max_fragmentation_loss = 70.312 MiB
20:22:07 0 D vector_length = 8500; fragmentation_loss = 8.750 KiB; max_fragmentation_loss = 66.406 MiB
20:22:07 0 D vector_length = 8000; fragmentation_loss = 12.656 KiB; max_fragmentation_loss = 62.500 MiB
20:22:07 0 D vector_length = 7500; fragmentation_loss = 16.562 KiB; max_fragmentation_loss = 58.594 MiB
20:22:07 0 D vector_length = 7000; fragmentation_loss = 20.469 KiB; max_fragmentation_loss = 54.688 MiB
20:22:07 0 D vector_length = 6500; fragmentation_loss = 24.375 KiB; max_fragmentation_loss = 50.781 MiB
20:22:07 0 D vector_length = 6000; fragmentation_loss = 28.281 KiB; max_fragmentation_loss = 46.875 MiB
20:22:07 0 D vector_length = 5500; fragmentation_loss = 32.188 KiB; max_fragmentation_loss = 42.969 MiB
20:22:07 0 D vector_length = 5000; fragmentation_loss = 36.094 KiB; max_fragmentation_loss = 39.062 MiB
20:22:07 0 D vector_length = 4500; fragmentation_loss = 40.000 KiB; max_fragmentation_loss = 35.156 MiB
20:22:07 0 D vector_length = 4000; fragmentation_loss = 43.906 KiB; max_fragmentation_loss = 31.250 MiB
20:22:07 0 D vector_length = 3500; fragmentation_loss = 47.812 KiB; max_fragmentation_loss = 27.344 MiB
20:22:07 0 D vector_length = 3000; fragmentation_loss = 51.719 KiB; max_fragmentation_loss = 23.438 MiB
20:22:07 0 D vector_length = 2500; fragmentation_loss = 55.625 KiB; max_fragmentation_loss = 19.531 MiB
20:22:07 0 D vector_length = 2000; fragmentation_loss = 59.531 KiB; max_fragmentation_loss = 16.000 MiB
20:22:07 0 D vector_length = 1500; fragmentation_loss = 63.438 KiB; max_fragmentation_loss = 16.000 MiB
20:22:07 0 D vector_length = 1000; fragmentation_loss = 67.344 KiB; max_fragmentation_loss = 16.000 MiB
20:22:07 0 D vector_length = 500; fragmentation_loss = 71.250 KiB; max_fragmentation_loss = 16.000 MiB
20:22:07 0 D vector_length = 0; fragmentation_loss = 75.156 KiB; max_fragmentation_loss = 16.000 MiB
20:22:07 0 D contents_sum = 333283335000; pop_sum = 333283335000
20:22:07 0 D end_info = { allocations: 12439 - 12439 = 0, requested: 9.528 MiB - 9.528 MiB = 0 B, allocated: 10.907 MiB - 10.907 MiB = 0 B, pages: 2223 - 2023 = 200, loss: 800.000 KiB = 100.000% }
20:22:07 0 D end_info_diff = { allocations: 10031 - 10031 = 0, requested: 462.930 KiB - 462.930 KiB = 0 B, allocated: 462.930 KiB - 462.930 KiB = 0 B, pages: 113 - 94 = 19, loss: 76.000 KiB = 100.000% }
20:22:07 0 D vector_length = 0; fragmentation_loss = 76.000 KiB; max_fragmentation_loss = 16.000 MiB
3_allocator_1_big::stress-------------------------- [passed]
20:22:07 0 I exit qemu; exit_code = ExitCode(SUCCESS)
```
