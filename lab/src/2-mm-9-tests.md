## Проверьте себя

После того как вы полностью реализуете работу с памятью, проверьте себя.
Запустите модульные и интеграционные тесты командой `make test` в корне репозитория.
Вы должны увидеть что-то вроде

```console
$ make test
...
running 1 test
test memory::block::test::block ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
...
kernel::tests::memory::basic_frame_allocator_functions------
14:30:35 0 D frames = [Frame(12492 @ 0p30CC000), Frame(12493 @ 0p30CD000)]
14:30:35 0 D reallocate_last_freed_frame = Frame(12492 @ 0p30CC000)
kernel::tests::memory::basic_frame_allocator_functions [passed]

kernel::tests::memory::allocated_frames_are_unique----------
14:30:35 0 D free_frames = 11436
14:30:35 0 D free_frames = 0
14:30:35 0 D free_frames = 11436
kernel::tests::memory::allocated_frames_are_unique- [passed]

kernel::tests::memory::allocate_page------------------------
14:30:35 0 D page = Page(4503599627370148 @ 0vFFFFFFFFFFEA4000)
kernel::tests::memory::allocate_page--------------- [passed]

kernel::tests::memory::shared_memory------------------------
14:30:35 0 D frame = Frame(12514 @ 0p30E2000)
14:30:35 0 D pages = [Page(4503599627370146 @ 0vFFFFFFFFFFEA2000), Page(4503599627370147 @ 0vFFFFFFFFFFEA3000)]
14:30:35 0 D write_ptr = 0xffffffffffea3000; read_ptr = 0xffffffffffea2000
14:30:35 0 D write_value = 0; read_value = 0
14:30:35 0 D write_value = 1; read_value = 1
14:30:35 0 D write_value = 2; read_value = 2
kernel::tests::memory::shared_memory--------------- [passed]
...
14:30:35 0 I exit qemu; exit_code = SUCCESS
```

В случае ошибки вы увидите что-нибудь вроде
```console
$ make test
...
panicked at 'called `Result::unwrap()` on an `Err` value: NoFrame', src/memory/main_frame_allocator.rs:49:14
--------------------------------------------------- [failed]
21:41:55 0 I exit qemu; exit_code = FAILURE
error: test failed, to rerun pass '--bin kernel'
```

Но учтите, что тесты не могут покрывать всё.
Так что даже если они прошли успешно, возможно код содержит ошибки,
которые проявят себя позже в следующих лабораторных работах.
Чтобы повысить свою уверенность, что этого не произойдёт,
добавьте собственные тесты, а также внимательно перечитайте свой код и код вокруг.
