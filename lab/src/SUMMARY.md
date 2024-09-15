# Summary

[Введение в лабораторные работы](0-intro-1-nikka.md)
[Почему Rust](0-intro-2-rust.md)
[Структура кода](0-intro-3-dirs.md)
[Компиляция и запуск тестов](0-intro-4-install.md)
[Отладчик, дизассемблирование, трассировки стека](0-intro-5-gdb.md)
[Журналирование и файл журнала](0-intro-6-log.md)
[Настройка VSCode](0-intro-7-vscode.md)
[Ссылки](0-intro-8-links.md)

# Лабораторная работа №1

- [Время, прерывания и конкурентный код](1-hw-0-intro.md)
  - [Часы реального времени](1-hw-1-rtc.md)
  - [Счётчик тактов процессора](1-hw-2-tsc.md)
  - [Блокировки](5-sync-0-intro.md)
    - [Once lock](5-sync-1-once-lock.md)
    - [Спин-блокировка](5-sync-2-spinlock.md)
    - [Sequence lock](5-sync-3-sequence-lock.md)
  - [Счётчики тиков](5-sync-4-correlation-point.md)
  - [Измерение частоты процессора и повышение разрешения часов](1-hw-3-high-resolution-date.md)
  - [Обработка прерываний RTC](1-hw-4-interrupts.md)
  - [Информация о системе](1-hw-5-info.md)
  - [Собираем всё вместе](1-hw-6-summary.md)

# Лабораторная работа №2

- [Память](2-mm-0-intro.md)
  - [Типы](2-mm-0-intro-1-types.md)
  - [Страничные отображения](2-mm-0-intro-2-mapping.md)
  - [Диаграммы преобразований](2-mm-0-intro-3-diagrams.md)
  - [План](2-mm-0-intro-4-plan.md)
  - [Временный аллокатор фреймов](2-mm-1-boot-frame-allocator.md)
  - [Виртуальное адресное пространство](2-mm-2-address-space.md)
    - [Аллокатор виртуальных страниц адресного пространства](2-mm-2-page-allocator.md)
    - [Отображение виртуальных страниц в физические фреймы](2-mm-4-translate.md)
    - [Высокоуровневый интерфейс управления адресным пространством](2-mm-5-map.md)
  - [Основной аллокатор фреймов](2-mm-8-main-frame-allocator.md)
  - [Проверьте себя](2-mm-9-tests.md)

# Лабораторная работа №3

- [Аллокатор памяти общего назначения и SMP](3-smp-0-intro.md)
  - [Аллокатор памяти общего назначения](3-allocator-0-intro.md)
    - [Аллокатор больших блоков памяти](3-allocator-1-big.md)
    - [Аллокатор маленьких блоков памяти](3-allocator-5-small.md)
  - [Поддержка нескольких процессоров (SMP)](3-smp-2-smp-0-intro.md)
    - [Работа с local APIC](4-process-1-local-apic.md)
    - [Состояние каждого процессора](4-process-2-cpus.md)
    - [Загрузка Application Processor](5-sync-5-ap-init.md)

# Лабораторная работа №4

- [Процессы](4-process-0-intro.md)
  - [Загрузка процесса в память](4-process-3-elf.md)
  - [Проверки доступа процесса к памяти](4-process-4-permission-checks.md)
  - [Переход в режим пользователя](4-process-5-user-mode.md)
  - [Поддержка системных вызовов](4-process-6-syscall.md)
  - [Таблица процессов](4-process-7-table.md)
  - [Вытесняющая многозадачность](4-process-8-preemption.md)
  - [Планировщик](4-process-9-scheduler.md)

# Лабораторная работа №5

- [Продвинутая работа с памятью в пространстве пользователя](6-um-0-intro.md)
  - [Разделяемая память](6-um-1-pipe.md)
  - [Системные вызовы для работы с виртуальной памятью](6-um-2-memory-syscalls.md)
  - [Eager fork](6-um-3-eager-fork.md)
  - [Обработка исключений в режиме пользователя](6-um-4-trap-handler.md)
  - [Copy-on-write fork](6-um-5-cow-fork.md)

# Лабораторная работа №6

- [Файловая система](7-fs-0-intro.md)
  - [Блочный кэш](7-fs-1-block-cache.md)
  - [Битовые карты занятых блоков и inode](7-fs-2-bitmap.md)
  - [Index node (inode)](7-fs-3-inode.md)
  - [Операции с директориями](7-fs-4-directory.md)
  - [Поиск файла по пути](7-fs-5-open.md)
  - [Вытеснение блоков из блочного кэша](7-fs-6-lru.md)
