## Битовая карта фиксированного размера


Для отслеживания какие именно элементы заняты, а какие свободны, служит структура
[`ku::collections::bitmap::Bitmap`]().

{{#include ../../doc/ku/collections/bitmap/struct.Bitmap.html.declaration}}

В этой лабораторной работе интересующими нас элементами будут блоки памяти аллокатора.
А в одной из последующих лабораторных работ --- блоки и
[inode](https://en.wikipedia.org/wiki/Inode)
файловой системы.

Каждое значение в срезе
[`Bitmap::bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html#structfield.bitmap)
отвечает за 64 элемента.
Элементу с меньшим номером соответствует бит с меньшим весом в соответствующем элементе.
Элемент свободен тогда и только тогда, когда соответствующий ему бит равен `0`.


### Задача 2 --- выделение свободного элемента с помощью битовой карты

Реализуйте метод
[`Bitmap::allocate()`](../../doc/ku/collections/bitmap/struct.Bitmap.html#method.allocate)
в файле
[`ku/src/collections/bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/src/collections/bitmap.rs).

{{#include ../../doc/ku/collections/bitmap/struct.Bitmap.html.method.allocate}}

При реализации учтите пару моментов:

- Искать каждый раз с самого начала [`Bitmap::bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html#structfield.bitmap) не эффективно. Лучше, например, обходить его по циклу с того места, на котором остановились в прошлый раз. Для хранения этой позиции между вызовами [`Bitmap::allocate()`](../../doc/ku/collections/bitmap/struct.Bitmap.html#method.allocate) служит поле [`Bitmap::cursor`](../../doc/ku/collections/bitmap/struct.Bitmap.html#structfield.cursor).
- Проверять каждый бит тоже не эффективно. Лучше сначала найти значение в [`Bitmap::bitmap`](../../doc/ku/collections/bitmap/struct.Bitmap.html#structfield.bitmap), в котором есть хотя бы один свободный бит. Поэтому биты сгруппированы именно по [`u64`](https://doc.rust-lang.org/nightly/core/primitive.u64.html), а не например по [`u8`](https://doc.rust-lang.org/nightly/core/primitive.u8.html).


### Проверьте себя

Теперь должны заработать тесты в файле
[`ku/tests/3-allocator-2-bitmap.rs`](https://gitlab.com/sergey-v-galtsev/nikka-public/-/blob/master/ku/tests/3-allocator-2-bitmap.rs):

```console
$ (cd ku && cargo test --test 3-allocator-2-bitmap)
...
running 3 tests
2025-11-02T19:28:33.242344Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[28, 29, 30, 31, 32]
2025-11-02T19:28:33.242368Z : allocation block=0 free=0
2025-11-02T19:28:33.242373Z : allocation reallocated_head=[0, 2, 4, 6, 8] reallocated_tail=[24, 26, 28, 30, 32]
2025-11-02T19:28:33.242427Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[61, 62, 63, 64, 65]
2025-11-02T19:28:33.242431Z : allocation block=0 free=0
2025-11-02T19:28:33.242435Z : allocation reallocated_head=[0, 2, 4, 6, 8] reallocated_tail=[56, 58, 60, 62, 64]
2025-11-02T19:28:33.242526Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[94, 95, 96, 97, 98]
2025-11-02T19:28:33.242531Z : allocation block=0 free=0
2025-11-02T19:28:33.242534Z : allocation reallocated_head=[64, 66, 68, 70, 72] reallocated_tail=[54, 56, 58, 60, 62]
2025-11-02T19:28:33.242680Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[127, 128, 129, 130, 131]
2025-11-02T19:28:33.242685Z : allocation block=0 free=0
2025-11-02T19:28:33.242688Z : allocation block=100 free=50
2025-11-02T19:28:33.242691Z : allocation reallocated_head=[64, 66, 68, 70, 72] reallocated_tail=[54, 56, 58, 60, 62]
2025-11-02T19:28:33.242861Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[160, 161, 162, 163, 164]
2025-11-02T19:28:33.242866Z : allocation block=0 free=0
2025-11-02T19:28:33.242869Z : allocation block=100 free=50
2025-11-02T19:28:33.242872Z : allocation reallocated_head=[128, 130, 132, 134, 136] reallocated_tail=[118, 120, 122, 124, 126]
2025-11-02T19:28:33.243099Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[193, 194, 195, 196, 197]
2025-11-02T19:28:33.243105Z : allocation block=0 free=0
2025-11-02T19:28:33.243107Z : allocation block=100 free=50
2025-11-02T19:28:33.243111Z : allocation reallocated_head=[128, 130, 132, 134, 136] reallocated_tail=[118, 120, 122, 124, 126]
2025-11-02T19:28:33.243386Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[226, 227, 228, 229, 230]
2025-11-02T19:28:33.243393Z : allocation block=0 free=0
2025-11-02T19:28:33.243395Z : allocation block=100 free=50
2025-11-02T19:28:33.243398Z : allocation block=200 free=100
2025-11-02T19:28:33.243401Z : allocation reallocated_head=[192, 194, 196, 198, 200] reallocated_tail=[182, 184, 186, 188, 190]
2025-11-02T19:28:33.243721Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[259, 260, 261, 262, 263]
2025-11-02T19:28:33.243728Z : allocation block=0 free=0
2025-11-02T19:28:33.243731Z : allocation block=100 free=50
2025-11-02T19:28:33.243734Z : allocation block=200 free=100
2025-11-02T19:28:33.243737Z : allocation reallocated_head=[192, 194, 196, 198, 200] reallocated_tail=[182, 184, 186, 188, 190]
2025-11-02T19:28:33.244072Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[292, 293, 294, 295, 296]
2025-11-02T19:28:33.244079Z : allocation block=0 free=0
2025-11-02T19:28:33.244081Z : allocation block=100 free=50
2025-11-02T19:28:33.244084Z : allocation block=200 free=100
2025-11-02T19:28:33.244087Z : allocation reallocated_head=[256, 258, 260, 262, 264] reallocated_tail=[246, 248, 250, 252, 254]
2025-11-02T19:28:33.244450Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[325, 326, 327, 328, 329]
2025-11-02T19:28:33.244457Z : allocation block=0 free=0
2025-11-02T19:28:33.244459Z : allocation block=100 free=50
2025-11-02T19:28:33.244462Z : allocation block=200 free=100
2025-11-02T19:28:33.244465Z : allocation block=300 free=150
2025-11-02T19:28:33.244468Z : allocation reallocated_head=[256, 258, 260, 262, 264] reallocated_tail=[246, 248, 250, 252, 254]
2025-11-02T19:28:33.244860Z : allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[358, 359, 360, 361, 362]
2025-11-02T19:28:33.244868Z : allocation block=0 free=0
2025-11-02T19:28:33.244871Z : allocation block=100 free=50
2025-11-02T19:28:33.244873Z : allocation block=200 free=100
2025-11-02T19:28:33.244876Z : allocation block=300 free=150
2025-11-02T19:28:33.244880Z : allocation reallocated_head=[320, 322, 324, 326, 328] reallocated_tail=[310, 312, 314, 316, 318]
2025-11-02T19:28:33.244888Z : allocation_complexity len=100 elapsed=893ns amortized_allocation_time=8ns
2025-11-02T19:28:33.245338Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[391, 392, 393, 394, 395]
2025-11-02T19:28:33.245354Z :            allocation block=0 free=0
2025-11-02T19:28:33.245364Z :            allocation block=100 free=50
2025-11-02T19:28:33.246853Z :            allocation block=200 free=100
2025-11-02T19:28:33.246868Z :            allocation block=300 free=150
2025-11-02T19:28:33.246881Z :            allocation reallocated_head=[320, 322, 324, 326, 328] reallocated_tail=[310, 312, 314, 316, 318]
2025-11-02T19:28:33.245998Z : allocation_complexity len=1000 elapsed=6.179µs amortized_allocation_time=6ns
2025-11-02T19:28:33.247340Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[424, 425, 426, 427, 428]
2025-11-02T19:28:33.247356Z :            allocation block=0 free=0
2025-11-02T19:28:33.247366Z :            allocation block=100 free=50
2025-11-02T19:28:33.247375Z :            allocation block=200 free=100
2025-11-02T19:28:33.247384Z :            allocation block=300 free=150
2025-11-02T19:28:33.247393Z :            allocation block=400 free=200
2025-11-02T19:28:33.247404Z :            allocation reallocated_head=[384, 386, 388, 390, 392] reallocated_tail=[374, 376, 378, 380, 382]
2025-11-02T19:28:33.247888Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[457, 458, 459, 460, 461]
2025-11-02T19:28:33.247905Z :            allocation block=0 free=0
2025-11-02T19:28:33.247914Z :            allocation block=100 free=50
2025-11-02T19:28:33.247924Z :            allocation block=200 free=100
2025-11-02T19:28:33.247933Z :            allocation block=300 free=150
2025-11-02T19:28:33.247942Z :            allocation block=400 free=200
2025-11-02T19:28:33.247953Z :            allocation reallocated_head=[384, 386, 388, 390, 392] reallocated_tail=[374, 376, 378, 380, 382]
2025-11-02T19:28:33.248126Z : allocation_complexity len=10000 elapsed=67.561µs amortized_allocation_time=6ns
2025-11-02T19:28:33.248458Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[490, 491, 492, 493, 494]
2025-11-02T19:28:33.248668Z :            allocation block=0 free=0
2025-11-02T19:28:33.248705Z :            allocation block=100 free=50
2025-11-02T19:28:33.248738Z :            allocation block=200 free=100
2025-11-02T19:28:33.248783Z :            allocation block=300 free=150
2025-11-02T19:28:33.248834Z :            allocation block=400 free=200
2025-11-02T19:28:33.248848Z :            allocation reallocated_head=[448, 450, 452, 454, 456] reallocated_tail=[438, 440, 442, 444, 446]
2025-11-02T19:28:33.249407Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[523, 524, 525, 526, 527]
2025-11-02T19:28:33.249957Z :            allocation block=0 free=0
2025-11-02T19:28:33.249966Z :            allocation block=100 free=50
2025-11-02T19:28:33.249970Z :            allocation block=200 free=100
2025-11-02T19:28:33.249975Z :            allocation block=300 free=150
2025-11-02T19:28:33.249979Z :            allocation block=400 free=200
2025-11-02T19:28:33.249983Z :            allocation block=500 free=250
2025-11-02T19:28:33.249989Z :            allocation reallocated_head=[448, 450, 452, 454, 456] reallocated_tail=[438, 440, 442, 444, 446]
2025-11-02T19:28:33.250355Z : allocation_complexity len=100000 elapsed=1.038549ms amortized_allocation_time=10ns
2025-11-02T19:28:33.250571Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[556, 557, 558, 559, 560]
2025-11-02T19:28:33.250590Z :            allocation block=0 free=0
2025-11-02T19:28:33.250593Z :            allocation block=100 free=50
2025-11-02T19:28:33.250596Z :            allocation block=200 free=100
2025-11-02T19:28:33.250600Z :            allocation block=300 free=150
2025-11-02T19:28:33.250604Z :            allocation block=400 free=200
2025-11-02T19:28:33.250609Z :            allocation block=500 free=250
2025-11-02T19:28:33.250614Z :            allocation reallocated_head=[512, 514, 516, 518, 520] reallocated_tail=[502, 504, 506, 508, 510]
2025-11-02T19:28:33.251199Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[589, 590, 591, 592, 593]
2025-11-02T19:28:33.251218Z :            allocation block=0 free=0
2025-11-02T19:28:33.251221Z :            allocation block=100 free=50
2025-11-02T19:28:33.251224Z :            allocation block=200 free=100
2025-11-02T19:28:33.251228Z :            allocation block=300 free=150
2025-11-02T19:28:33.251231Z :            allocation block=400 free=200
2025-11-02T19:28:33.251235Z :            allocation block=500 free=250
2025-11-02T19:28:33.251240Z :            allocation reallocated_head=[512, 514, 516, 518, 520] reallocated_tail=[502, 504, 506, 508, 510]
2025-11-02T19:28:33.251857Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[622, 623, 624, 625, 626]
2025-11-02T19:28:33.251876Z :            allocation block=0 free=0
2025-11-02T19:28:33.251879Z :            allocation block=100 free=50
2025-11-02T19:28:33.251881Z :            allocation block=200 free=100
2025-11-02T19:28:33.251884Z :            allocation block=300 free=150
2025-11-02T19:28:33.251887Z :            allocation block=400 free=200
2025-11-02T19:28:33.251892Z :            allocation block=500 free=250
2025-11-02T19:28:33.251895Z :            allocation block=600 free=300
2025-11-02T19:28:33.251900Z :            allocation reallocated_head=[576, 578, 580, 582, 584] reallocated_tail=[566, 568, 570, 572, 574]
2025-11-02T19:28:33.252540Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[655, 656, 657, 658, 659]
2025-11-02T19:28:33.252559Z :            allocation block=0 free=0
2025-11-02T19:28:33.252562Z :            allocation block=100 free=50
2025-11-02T19:28:33.252564Z :            allocation block=200 free=100
2025-11-02T19:28:33.252568Z :            allocation block=300 free=150
2025-11-02T19:28:33.252571Z :            allocation block=400 free=200
2025-11-02T19:28:33.252574Z :            allocation block=500 free=250
2025-11-02T19:28:33.252577Z :            allocation block=600 free=300
2025-11-02T19:28:33.252581Z :            allocation reallocated_head=[576, 578, 580, 582, 584] reallocated_tail=[566, 568, 570, 572, 574]
2025-11-02T19:28:33.253228Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[688, 689, 690, 691, 692]
2025-11-02T19:28:33.253246Z :            allocation block=0 free=0
2025-11-02T19:28:33.253249Z :            allocation block=100 free=50
2025-11-02T19:28:33.253252Z :            allocation block=200 free=100
2025-11-02T19:28:33.253254Z :            allocation block=300 free=150
2025-11-02T19:28:33.253260Z :            allocation block=400 free=200
2025-11-02T19:28:33.253263Z :            allocation block=500 free=250
2025-11-02T19:28:33.253265Z :            allocation block=600 free=300
2025-11-02T19:28:33.253270Z :            allocation reallocated_head=[640, 642, 644, 646, 648] reallocated_tail=[630, 632, 634, 636, 638]
2025-11-02T19:28:33.253960Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[721, 722, 723, 724, 725]
2025-11-02T19:28:33.253980Z :            allocation block=0 free=0
2025-11-02T19:28:33.253983Z :            allocation block=100 free=50
2025-11-02T19:28:33.253985Z :            allocation block=200 free=100
2025-11-02T19:28:33.253989Z :            allocation block=300 free=150
2025-11-02T19:28:33.253992Z :            allocation block=400 free=200
2025-11-02T19:28:33.253998Z :            allocation block=500 free=250
2025-11-02T19:28:33.254000Z :            allocation block=600 free=300
2025-11-02T19:28:33.254003Z :            allocation block=700 free=350
2025-11-02T19:28:33.254008Z :            allocation reallocated_head=[640, 642, 644, 646, 648] reallocated_tail=[630, 632, 634, 636, 638]
2025-11-02T19:28:33.254732Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[754, 755, 756, 757, 758]
2025-11-02T19:28:33.254752Z :            allocation block=0 free=0
2025-11-02T19:28:33.254755Z :            allocation block=100 free=50
2025-11-02T19:28:33.254758Z :            allocation block=200 free=100
2025-11-02T19:28:33.254763Z :            allocation block=300 free=150
2025-11-02T19:28:33.254767Z :            allocation block=400 free=200
2025-11-02T19:28:33.254770Z :            allocation block=500 free=250
2025-11-02T19:28:33.254773Z :            allocation block=600 free=300
2025-11-02T19:28:33.254775Z :            allocation block=700 free=350
2025-11-02T19:28:33.254780Z :            allocation reallocated_head=[704, 706, 708, 710, 712] reallocated_tail=[694, 696, 698, 700, 702]
2025-11-02T19:28:33.255523Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[787, 788, 789, 790, 791]
2025-11-02T19:28:33.255543Z :            allocation block=0 free=0
2025-11-02T19:28:33.255549Z :            allocation block=100 free=50
2025-11-02T19:28:33.255552Z :            allocation block=200 free=100
2025-11-02T19:28:33.255555Z :            allocation block=300 free=150
2025-11-02T19:28:33.255557Z :            allocation block=400 free=200
2025-11-02T19:28:33.255562Z :            allocation block=500 free=250
2025-11-02T19:28:33.255566Z :            allocation block=600 free=300
2025-11-02T19:28:33.255570Z :            allocation block=700 free=350
2025-11-02T19:28:33.255577Z :            allocation reallocated_head=[704, 706, 708, 710, 712] reallocated_tail=[694, 696, 698, 700, 702]
2025-11-02T19:28:33.256348Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[820, 821, 822, 823, 824]
2025-11-02T19:28:33.256361Z :            allocation block=0 free=0
2025-11-02T19:28:33.256365Z :            allocation block=100 free=50
2025-11-02T19:28:33.256370Z :            allocation block=200 free=100
2025-11-02T19:28:33.256374Z :            allocation block=300 free=150
2025-11-02T19:28:33.256378Z :            allocation block=400 free=200
2025-11-02T19:28:33.256382Z :            allocation block=500 free=250
2025-11-02T19:28:33.256385Z :            allocation block=600 free=300
2025-11-02T19:28:33.256389Z :            allocation block=700 free=350
2025-11-02T19:28:33.256393Z :            allocation block=800 free=400
2025-11-02T19:28:33.256398Z :            allocation reallocated_head=[768, 770, 772, 774, 776] reallocated_tail=[758, 760, 762, 764, 766]
2025-11-02T19:28:33.257181Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[853, 854, 855, 856, 857]
2025-11-02T19:28:33.257201Z :            allocation block=0 free=0
2025-11-02T19:28:33.257204Z :            allocation block=100 free=50
2025-11-02T19:28:33.257207Z :            allocation block=200 free=100
2025-11-02T19:28:33.257210Z :            allocation block=300 free=150
2025-11-02T19:28:33.257215Z :            allocation block=400 free=200
2025-11-02T19:28:33.257218Z :            allocation block=500 free=250
2025-11-02T19:28:33.257220Z :            allocation block=600 free=300
2025-11-02T19:28:33.257223Z :            allocation block=700 free=350
2025-11-02T19:28:33.257225Z :            allocation block=800 free=400
2025-11-02T19:28:33.257231Z :            allocation reallocated_head=[768, 770, 772, 774, 776] reallocated_tail=[758, 760, 762, 764, 766]
2025-11-02T19:28:33.258051Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[886, 887, 888, 889, 890]
2025-11-02T19:28:33.258072Z :            allocation block=0 free=0
2025-11-02T19:28:33.258075Z :            allocation block=100 free=50
2025-11-02T19:28:33.258078Z :            allocation block=200 free=100
2025-11-02T19:28:33.258082Z :            allocation block=300 free=150
2025-11-02T19:28:33.258085Z :            allocation block=400 free=200
2025-11-02T19:28:33.258087Z :            allocation block=500 free=250
2025-11-02T19:28:33.258090Z :            allocation block=600 free=300
2025-11-02T19:28:33.258093Z :            allocation block=700 free=350
2025-11-02T19:28:33.258095Z :            allocation block=800 free=400
2025-11-02T19:28:33.258100Z :            allocation reallocated_head=[832, 834, 836, 838, 840] reallocated_tail=[822, 824, 826, 828, 830]
2025-11-02T19:28:33.258386Z : allocation_complexity len=1000000 elapsed=6.762638ms amortized_allocation_time=6ns
2025-11-02T19:28:33.259098Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[919, 920, 921, 922, 923]
2025-11-02T19:28:33.259115Z :            allocation block=0 free=0
2025-11-02T19:28:33.259119Z :            allocation block=100 free=50
2025-11-02T19:28:33.259122Z :            allocation block=200 free=100
2025-11-02T19:28:33.259124Z :            allocation block=300 free=150
2025-11-02T19:28:33.259127Z :            allocation block=400 free=200
2025-11-02T19:28:33.259129Z :            allocation block=500 free=250
2025-11-02T19:28:33.259132Z :            allocation block=600 free=300
2025-11-02T19:28:33.259135Z :            allocation block=700 free=350
2025-11-02T19:28:33.259137Z :            allocation block=800 free=400
2025-11-02T19:28:33.259140Z :            allocation block=900 free=450
2025-11-02T19:28:33.259147Z :            allocation reallocated_head=[832, 834, 836, 838, 840] reallocated_tail=[822, 824, 826, 828, 830]
2025-11-02T19:28:33.260020Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[952, 953, 954, 955, 956]
2025-11-02T19:28:33.260044Z :            allocation block=0 free=0
2025-11-02T19:28:33.260049Z :            allocation block=100 free=50
2025-11-02T19:28:33.260052Z :            allocation block=200 free=100
2025-11-02T19:28:33.260057Z :            allocation block=300 free=150
2025-11-02T19:28:33.260061Z :            allocation block=400 free=200
2025-11-02T19:28:33.260064Z :            allocation block=500 free=250
2025-11-02T19:28:33.260068Z :            allocation block=600 free=300
2025-11-02T19:28:33.260073Z :            allocation block=700 free=350
2025-11-02T19:28:33.260077Z :            allocation block=800 free=400
2025-11-02T19:28:33.260081Z :            allocation block=900 free=450
2025-11-02T19:28:33.260088Z :            allocation reallocated_head=[896, 898, 900, 902, 904] reallocated_tail=[886, 888, 890, 892, 894]
2025-11-02T19:28:33.260988Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[985, 986, 987, 988, 989]
2025-11-02T19:28:33.261004Z :            allocation block=0 free=0
2025-11-02T19:28:33.261008Z :            allocation block=100 free=50
2025-11-02T19:28:33.261012Z :            allocation block=200 free=100
2025-11-02T19:28:33.261016Z :            allocation block=300 free=150
2025-11-02T19:28:33.261020Z :            allocation block=400 free=200
2025-11-02T19:28:33.261023Z :            allocation block=500 free=250
2025-11-02T19:28:33.261027Z :            allocation block=600 free=300
2025-11-02T19:28:33.261031Z :            allocation block=700 free=350
2025-11-02T19:28:33.261035Z :            allocation block=800 free=400
2025-11-02T19:28:33.261039Z :            allocation block=900 free=450
2025-11-02T19:28:33.261046Z :            allocation reallocated_head=[896, 898, 900, 902, 904] reallocated_tail=[886, 888, 890, 892, 894]
2025-11-02T19:28:33.261977Z :            allocation allocated_head=[0, 1, 2, 3, 4] allocated_tail=[1018, 1019, 1020, 1021, 1022]
2025-11-02T19:28:33.261994Z :            allocation block=0 free=0
2025-11-02T19:28:33.261998Z :            allocation block=100 free=50
2025-11-02T19:28:33.262001Z :            allocation block=200 free=100
2025-11-02T19:28:33.262004Z :            allocation block=300 free=150
2025-11-02T19:28:33.262007Z :            allocation block=400 free=200
2025-11-02T19:28:33.262013Z :            allocation block=500 free=250
2025-11-02T19:28:33.262017Z :            allocation block=600 free=300
2025-11-02T19:28:33.262020Z :            allocation block=700 free=350
2025-11-02T19:28:33.262024Z :            allocation block=800 free=400
2025-11-02T19:28:33.262028Z :            allocation block=900 free=450
2025-11-02T19:28:33.262032Z :            allocation block=1000 free=500
2025-11-02T19:28:33.262039Z :            allocation reallocated_head=[960, 962, 964, 966, 968] reallocated_tail=[950, 952, 954, 956, 958]
test allocation ... ok
test basic ... ok
2025-11-02T19:28:33.325320Z : allocation_complexity len=10000000 elapsed=65.270453ms amortized_allocation_time=6ns
2025-11-02T19:28:33.325335Z : allocation_complexity min_time=6ns avg_time=7ns max_time=10ns
test allocation_complexity ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```


### Ориентировочный объём работ этой части лабораторки

```console
ku/src/collections/bitmap.rs | 40 ++++++++++++++++++++++++++++++++++++----
1 file changed, 36 insertions(+), 4 deletions(-)
```
