# execenv

`execenv` is a small Unix-oriented entrypoint wrapper for containers.

Choose your language:

- [English documentation](README.en.md)
- [Русская документация](README.ru.md)

---

`execenv` renders `${VAR}` placeholders in one or more files using values from the current environment. After rendering, it can replace itself with another process via `exec(2)`.

`execenv` подставляет значения переменных окружения в плейсхолдеры `${VAR}` в одном или нескольких файлах. После рендера утилита может заменить свой процесс другим процессом через `exec(2)`.
