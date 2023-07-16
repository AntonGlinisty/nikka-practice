#![deny(warnings)]

use ku::{
    log::debug,
    sync::Spinlock,
};

mod log;

#[should_panic(expected = "deadlock")]
#[test]
fn deadlock_in_panic_message() {
    recursive_deadlock()
}

#[should_panic(expected = "ku/tests/5-sync-2-deadlock.rs:38")]
#[test]
fn definition_line_number_in_panic_message() {
    recursive_deadlock()
}

#[should_panic(expected = "ku/tests/5-sync-2-deadlock.rs:39")]
#[test]
fn owner_line_number_in_panic_message() {
    recursive_deadlock()
}

#[should_panic(expected = "ku/tests/5-sync-2-deadlock.rs:42")]
#[test]
fn deadlock_line_number_in_panic_message() {
    recursive_deadlock()
}

/// Строки, ссылки на которые должны попасть в сообщение паники, отмечены комментарием `// (*)`.
fn recursive_deadlock() {
    log::init();

    let spinlock = Spinlock::new(0); // (*)
    let lock = spinlock.lock(); // (*)

    debug!(?spinlock, "attempting to lock a locked spinlock");
    drop(spinlock.lock()); // (*)
    debug!(error = "no panic occurred");

    drop(lock);
}
