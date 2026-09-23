### Канал для анонсов

Важные объявления про курс публикуются в канале: https://t.me/+6hudiffitLU5MTYy

### Чат курса

Вступайте в чат в Телеграм, он создан для обсуждения и оперативного решения любых вопросов, связанных с курсом:<br><br>
<a href="https://t.me/+PAXh6qX5Nzg1MDcy"><img src="https://storage.yandexcloud.net/lms-vault/media/gallery/Logo/IMG_5903_2_2.png"></a>

<hr>

*Если у вас что-то не получается, пишите свои вопросы в этот телеграм чат.*

### Настройка окружения

Для работы на курсе вам потребуется ноутбук с установленной операционной системой Linux.
Мы рекомендуем использовать Ubuntu 24.04 или более новую, так как там есть весь необходимый софт достаточно свежих версий.

#### Регистрация в системе

Зарегистрируйтесь в проверяющей системе. Если вы уже зарегистрировались в manytask.org на другом курсе, вместо этого шага зайдите на [app.manytask.org/os-2026-fall](https://app.manytask.org/os-2026-fall) и нажмите Login.

  - Заполните форму и зайдите в систему: [app.manytask.org/os-2026-fall](https://app.manytask.org/os-2026-fall). Secret code – `deadbeef`.
  - Сгенерируйте ssh ключ и добавьте его в систему.
    1. `$ ssh-keygen -N "" -f ~/.ssh/id_rsa` – сгенерирует пару файлов `id_rsa` и `id_rsa.pub`. (Если у вас уже есть ключ, этот шаг не нужен).
    2. Скопируйте содержимое файла id_rsa.pub (`cat ~/.ssh/id_rsa.pub`) в [https://gitlab.manytask.org/-/user_settings/ssh_keys](https://gitlab.manytask.org/-/user_settings/ssh_keys).
    3. Проверьте, что ssh ключ работает. Выполните команду `ssh git@gitlab.manytask.org`. Вы должны увидеть такое приветствие:

```
$ ssh git@gitlab.manytask.org
PTY allocation request failed on channel 0
Welcome to GitLab, Fedor Korotkiy!
Connection to gitlab.manytask.org closed.
```

#### Настройка репозитория

Склонируйте репозиторий с задачами:

```
git clone https://gitlab.com/sergey-v-galtsev/nikka-public nikka
```

Команда `git clone` создаст директорию `nikka` и запишет туда все файлы из этого репозитория.

Для отправки решения на сервер, необходимо, чтобы у вас были заданы имя и email в git:

```
git config --global user.name "Vasya Pupkin"
git config --global user.email vasya@pupkin.ru
```

- Откройте страницу своего приватного репозитория в браузере. Для этого перейдите по ссылке `My Repo` на странице с задачами [app.manytask.org/os-2026-fall](https://app.manytask.org/os-2026-fall).
- Добавьте в git свой приватный репозиторий. Для этого запустите из директории репозитория команду:

```
git remote add student $ADDRESS
```

`$ADDRESS` нужно скопировать со страницы репозитория. Синяя кнопка `Clone` -> `Clone with SSH`.
Выглядеть он будет примерно как `git@gitlab.manytask.org:os/students-2026-2027/XXXXXX.git`.

Каждую неделю после занятий вам надо будет обновлять репозиторий, чтобы у вас появились условия
новых задач:

```
git pull --rebase
```

#### Установка qemu, llvm и Rust

Для лабораторных работ нам понадобится эмулятор компьютера – `qemu`.
А для символизации раскруток стека нужен будет `llvm-symbolizer` из пакета `llvm`.
Поставьте их командой

```
sudo apt install qemu-system-x86 qemu-utils llvm gcc
```

Поставьте Rust nightly той версии, которая указана в `rust-toolchain.toml` репозитория.
Например, можно в корне репозитория выполнить команду

```
make install
```

Она поставит Rust в вашу домашнюю директорию.
При этом можно использовать конфигурацию по умолчанию –
выбрать пункт `1) Proceed with installation (default)`.
В любом случае, в конце-концов установится версия `nightly`, которая прописана в файле
`rust-toolchain.toml` репозитория.

**Переоткройте терминал, чтобы в `$PATH` стали доступны новые команды.**

#### Отправка решения

Чтобы проверить работоспособность окружения, решите первую тестовую задачу в директории `sem/hello_world`:

1. Из корневой директории репозитория с задачами перейдите в директорию с задачей – `cd sem/hello_world`.
1. Запустите команду `cargo test`. Она прогонит тесты, которые ожидаемо упадут. Вместо `cargo test` можно использовать её сокращение `cargo t`.
1. Откройте файл `src/lib.rs`.
1. Напишите правильную реализацию функции `sum` в `src/lib.rs`.
1. Снова запустите команду `cargo test`. На этот раз тест должен пройти.
1. Закомитьте изменения: `git commit -a -m 'Solve task: hello_world'`. Сообщения выбирайте такие, по которым потом сможете вспомнить что вы меняли в коммите.
1. Отправьте своё решение на сервер: `git push student HEAD:submit/TASK_NAME`, где `TASK_NAME` нужно заменить на название задачи (`hello_world`). Название можно взять из таблицы gdoc или с сайта manytask.org. Ваш сабмит должен появиться по ссылке `My Submits` на [app.manytask.org/os-2026-fall](https://app.manytask.org/os-2026-fall).
1. После успешного прохождения тестов вам должно начислиться 100 баллов в таблице с баллами.
