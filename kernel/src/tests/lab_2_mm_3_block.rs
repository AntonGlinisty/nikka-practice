use kernel::{
    Subsystems,
    log::debug,
    memory::{
        Block,
        Virt,
    },
};

mod init;

init!(Subsystems::empty());

#[test_case]
fn block() {
    for start in 0 .. 100 {
        for count in 0 .. 100 {
            let end = start + count;
            let mut block = Block::<Virt>::from_index(start, end).unwrap();

            if start % 25 == 0 && end % 33 == 0 {
                debug!(start, end, %block);
            }

            assert_eq!(block.count(), count);

            for requested_count in count + 1 .. count + 3 {
                assert!(block.tail(requested_count).is_none());
            }

            let mut requested_count = 0;

            while requested_count < block.count() {
                let old_count = block.count();
                let old_end = block.end_address().unwrap().into_usize();

                block.tail(0).unwrap();
                assert_eq!(block.count(), old_count);

                let b = block.tail(requested_count).unwrap();

                assert_eq!(block.count(), old_count - requested_count);
                assert_eq!(b.count(), requested_count);

                assert_eq!(block.start_address().into_usize(), start);
                assert!(
                    block.start_address().into_usize() < block.end_address().unwrap().into_usize(),
                );
                assert_eq!(
                    block.start_address().into_usize() + old_count - requested_count,
                    block.end_address().unwrap().into_usize(),
                );
                assert_eq!(
                    block.end_address().unwrap().into_usize(),
                    b.start_address().into_usize(),
                );
                assert!(b.start_address().into_usize() <= b.end_address().unwrap().into_usize());
                assert_eq!(
                    b.start_address().into_usize() + requested_count,
                    b.end_address().unwrap().into_usize(),
                );
                assert!(b.end_address().unwrap().into_usize() <= end);
                assert_eq!(b.end_address().unwrap().into_usize(), old_end);

                requested_count += 1;
            }

            if count > 0 {
                let remaining_count = block.count();
                assert!(remaining_count > 0);

                let c = block.tail(remaining_count).unwrap();

                assert_eq!(block.count(), 0);
                assert_eq!(c.count(), remaining_count);

                assert_eq!(block.start_address().into_usize(), start);
                assert_eq!(
                    block.start_address().into_usize(),
                    block.end_address().unwrap().into_usize(),
                );
                assert_eq!(
                    block.end_address().unwrap().into_usize(),
                    c.start_address().into_usize(),
                );
                assert!(c.start_address().into_usize() < c.end_address().unwrap().into_usize());
                assert_eq!(
                    c.start_address().into_usize() + remaining_count,
                    c.end_address().unwrap().into_usize(),
                );
                assert!(c.end_address().unwrap().into_usize() <= end);
            }

            block.tail(0).unwrap();
            assert_eq!(block.count(), 0);
            assert!(block.tail(1).is_none());
        }
    }
}
