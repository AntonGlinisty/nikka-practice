use std::vec;

use rop::{
    self,
    ropchain,
};

#[test]
fn ropchain() {
    let gadgets = rop::gadgets();
    let mut ropchain = vec![0; 1024];

    let mut len = ropchain::make_ropchain(&mut ropchain).len();

    for gadget in ropchain.iter() {
        assert!(
            *gadget == 0 || gadgets.contains(gadget),
            "ropchain verification failed, no such gadget: {}",
            *gadget,
        );
    }

    // Позиция адреса `return_from_ropchain`, куда делается последний `ret` при запуске `ropchain`.
    // См. `rop::run_ropchain()`.
    len += 1;

    let expected = 0xDEAD_BEEF;
    let result = rop::run_ropchain(&ropchain[.. len]);

    assert_eq!(
        result, expected,
        "wrong result {result:#X}, expected {expected:#X}",
    );
}
