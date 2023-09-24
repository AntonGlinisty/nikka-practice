use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use context::{
    Context,
    ContextIs,
    load,
    save,
};

#[test]
fn simple() {
    let mut ctx = Context::new();

    println!("");
    println!("    saving ctx");
    if unsafe { save(&mut ctx) } == ContextIs::Saved {
        println!("    loading ctx");
        unsafe {
            load(&ctx);
        }
    }
    println!("    ctx loaded");
}

#[test]
fn chain() {
    let mut ctx0 = Context::new();
    let mut ctx1 = Context::new();
    let mut ctx2 = Context::new();
    let i = AtomicUsize::new(0);

    println!("");
    println!("    saving ctx0");
    if unsafe { save(&mut ctx0) } == ContextIs::Loaded {
        i.fetch_add(1, Ordering::Relaxed);
        println!("    loading ctx1");
        unsafe {
            load(&ctx1);
        }
    }

    println!("    saving ctx1");
    if unsafe { save(&mut ctx1) } == ContextIs::Loaded {
        i.fetch_add(1, Ordering::Relaxed);
        println!("    loading ctx2");
        unsafe {
            load(&ctx2);
        }
    }

    println!("    saving ctx2");
    if unsafe { save(&mut ctx2) } == ContextIs::Saved {
        i.fetch_add(1, Ordering::Relaxed);
        println!("    loading ctx0");
        unsafe {
            load(&ctx0);
        }
    }

    assert_eq!(i.load(Ordering::Relaxed), 3);
}

#[test]
fn context_loop() {
    let i = AtomicUsize::new(0);
    let mut ctx0 = Context::new();

    println!("");
    println!("    saving ctx");
    if unsafe { save(&mut ctx0) } == ContextIs::Saved {
        println!("    loading ctx i={}", i.load(Ordering::Relaxed));
        unsafe {
            load(&ctx0);
        }
    } else if i.load(Ordering::Relaxed) < 8 {
        i.fetch_add(1, Ordering::Relaxed);
        println!("    loading ctx i={}", i.load(Ordering::Relaxed));
        unsafe {
            load(&ctx0);
        }
    }

    assert_eq!(i.load(Ordering::Relaxed), 8);
}

#[inline(never)]
fn load1(ctx: &Context) {
    println!("    inside load1");
    unsafe {
        load(ctx);
    }
}

#[inline(never)]
fn load0(ctx: &Context) {
    println!("    inside load0");
    load1(ctx);
}

#[test]
fn inside_function() {
    let mut ctx = Context::new();

    println!("");
    println!("    saving ctx");
    if unsafe { save(&mut ctx) } == ContextIs::Saved {
        load0(&ctx);
    }
    println!("    loaded ctx");
}
