fn memory_allocator_basic() {
    let start_info = allocator::info();
    debug!(%start_info);

    {
        let mut a = Box::new(1_i64);
        let a_ptr = &mut *a as *mut i64;
        unsafe {
            a_ptr.write_volatile(a_ptr.read_volatile() + 1);
        }
        debug!(box_contents = *a);
        my_assert!(*a == 2);

        let info = allocator::info();
        let requested = mem::size_of_val(&*a);
        let info_diff = (info - start_info).unwrap();
        debug!(%info);
        debug!(%info_diff);

        my_assert!(info_diff.allocations().positive() == 1);
        my_assert!(info_diff.allocations().negative() == 0);
        my_assert!(info_diff.allocated().balance() >= requested);
        my_assert!(info_diff.pages().positive() == 1);
        my_assert!(info_diff.pages().negative() == 0);
    }

    let end_info = allocator::info();
    let end_info_diff = (end_info - start_info).unwrap();
    debug!(%end_info);
    debug!(%end_info_diff);

    my_assert!(end_info_diff.allocations().positive() == 1);
    my_assert!(end_info_diff.allocations().negative() == 1);
    my_assert!(end_info_diff.requested().balance() == 0);
    my_assert!(end_info_diff.allocated().balance() == 0);
    my_assert!(end_info_diff.pages().positive() == 1);
    my_assert!(end_info_diff.pages().negative() == 1);
    my_assert!(end_info_diff.fragmentation_loss() == 0);
    my_assert!(end_info_diff.fragmentation_loss_percentage() == 0.0);

    my_assert!(end_info.allocations().balance() == 0);
    my_assert!(end_info.pages().balance() == 0);
    my_assert!(end_info.fragmentation_loss() == 0);
}

fn memory_allocator_grow_and_shrink() {
    let start_info = allocator::info();
    debug!(%start_info);

    let mut vec = Vec::new();
    let mut push_sum = 0;

    for a in 1 .. 3 * Page::SIZE {
        vec.push(a);
        my_assert!(vec.len() == a);
        push_sum += a;
    }

    let contents_sum = vec.iter().sum::<usize>();
    debug!(contents_sum, push_sum);
    my_assert!(contents_sum == push_sum);

    let info = allocator::info();
    let info_diff = (info - start_info).unwrap();
    debug!(%info);
    debug!(%info_diff);
    my_assert!(info_diff.fragmentation_loss() == 0);
    my_assert!(info_diff.fragmentation_loss_percentage() <= 0.0);

    let mut pop_sum = 0;

    while !vec.is_empty() {
        pop_sum += vec.pop().unwrap();
        if vec.len() <= vec.capacity() / 2 {
            vec.shrink_to_fit();
        }
    }

    debug!(contents_sum, pop_sum);
    my_assert!(contents_sum == pop_sum);

    let end_info = allocator::info();
    debug!(%end_info);
    my_assert!(end_info.allocations().balance() == 0);
    my_assert!(end_info.pages().balance() == 0);
    my_assert!(end_info.fragmentation_loss() == 0);
}
