/// [Битовая карта](https://en.wikipedia.org/wiki/Free-space_bitmap)
/// фиксированного размера
/// для отслеживания какие именно элементы заняты, а какие --- свободны.
mod bitmap;

/// LRU--кэш
/// ([Least Recently Used](https://en.wikipedia.org/wiki/Cache_replacement_policies#LRU)) ---
/// кэш с реализацией алгоритма вытеснения давно неиспользуемых данных.
mod lru;

pub use bitmap::Bitmap;
pub use lru::Lru;
