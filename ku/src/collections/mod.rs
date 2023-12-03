/// LRU--кэш
/// ([Least Recently Used](https://en.wikipedia.org/wiki/Cache_replacement_policies#LRU)) ---
/// кэш с реализацией алгоритма вытеснения давно неиспользуемых данных.
mod lru;

pub use lru::Lru;
