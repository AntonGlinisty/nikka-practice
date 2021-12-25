#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#![feature(asm)]
#![no_main]
#![no_std]

use heapless::{
    String,
    Vec,
};

use ku::{
    error::Result,
    log::info,
    memory::{
        Block,
        Page,
        Virt,
        mmu::{
            PAGE_OFFSET_BITS,
            PAGE_TABLE_INDEX_BITS,
            PAGE_TABLE_LEAF_LEVEL,
            PAGE_TABLE_ROOT_LEVEL,
            PageTableFlags,
            USER,
            USER_RW,
        },
    },
    process,
    process::Pid,
};

use lib::{
    entry,
    memory,
    syscall,
};

entry!(main);

fn main() {
    let mut pedigree = Vec::<Pid, DEPTH>::new();

    let mut name = String::<MAX_NAME>::new();
    name.push_str("eager_fork ").unwrap();

    fork_tree(&mut pedigree, &mut name, '*');
}

fn fork_tree(
    pedigree: &mut Vec<Pid, DEPTH>,
    name: &mut String<MAX_NAME>,
    suffix: char,
) {
    name.push(suffix).unwrap();
    pedigree.push(process::pid()).unwrap();

    info!(
        name = name.as_str(),
        ?pedigree,
        len = pedigree.len(),
        capacity = pedigree.capacity(),
    );

    for child in '0' .. '3' {
        if pedigree.len() < pedigree.capacity() && eager_fork().expect("failed to eager_fork()") {
            fork_tree(pedigree, name, child);
        }
    }
}

fn eager_fork() -> Result<bool> {
    // TODO: your code here.
    unimplemented!();
}

fn copy_address_space(child: Pid) -> Result<()> {
    copy_page_table(child, PAGE_TABLE_ROOT_LEVEL, Virt::default())
}

fn copy_page_table(
    child: Pid,
    level: u32,
    start: Virt,
) -> Result<()> {
    // TODO: your code here.
    unimplemented!();
}

const DEPTH: usize = 3;
const MAX_NAME: usize = 64;
