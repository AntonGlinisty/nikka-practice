# Summary

[Введение в лабораторные работы](0-intro-1-nikka.md)
[Почему Rust](0-intro-2-rust.md)
[Структура кода](0-intro-3-dirs.md)
[Компиляция и запуск тестов](0-intro-4-install.md)
[Запуск с отладчиком gdb](0-intro-5-gdb.md)
[Журналирование и файл журнала](0-intro-6-log.md)

# Лабораторная работа №1

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

# Лабораторная работа №2

- [Процессы](4-process-0-intro.md)
  - [Загрузка процесса в память](4-process-3-elf.md)
  - [Проверки доступа процесса к памяти](4-process-4-permission-checks.md)
  - [Переход в режим пользователя](4-process-5-user-mode.md)
  - [Поддержка системных вызовов](4-process-6-syscall.md)

# Лабораторная работа №3

- [Конкурентное выполнение задач](3-smp-0-intro.md)
  - [Поддержка нескольких процессоров (SMP)](3-smp-2-smp-0-intro.md)
    - [Работа с local APIC](4-process-1-local-apic.md)
    - [Состояние каждого процессора](4-process-2-cpus.md)
    - [Загрузка Application Processor](5-sync-5-ap-init.md)
  - [Таблица процессов](4-process-7-table.md)
  - [Вытесняющая многозадачность](4-process-8-preemption.md)
  - [Планировщик](4-process-9-scheduler.md)

# Лабораторная работа №4

- [Продвинутая работа с памятью в пространстве пользователя](6-um-0-intro.md)
  - [Разделяемая память](6-um-1-pipe.md)
  - [Системные вызовы для работы с виртуальной памятью](6-um-2-memory-syscalls.md)
  - [Eager fork](6-um-3-eager-fork.md)
  - [Обработка исключений в режиме пользователя](6-um-4-trap-handler.md)
  - [Copy-on-write fork](6-um-5-cow-fork.md)
