.intel_syntax noprefix

.text

.global save
.global load


/// Записывает контекст в структуру `Context`, указатель на которую принимает в регистре RDI.
/// Возвращает в регистре RAX результат `0`.
save:
    // TODO: your code here.


/// Прыгает на контекст из структуры `Context`, указатель на которую принимает в регистре RDI.
/// Возвращает в регистре RAX результат `1`.
load:
    // TODO: your code here.
