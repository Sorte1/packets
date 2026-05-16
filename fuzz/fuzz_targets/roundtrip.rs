#![no_main]

use libfuzzer_sys::fuzz_target;
use packet_core::wire::{repack_frame, unpack_frame};

fuzz_target!(|data: &[u8]| {
    let Ok((values, trailer)) = unpack_frame(data) else {
        return;
    };
    let packed = repack_frame(&values, trailer).unwrap();
    let (values2, trailer2) = unpack_frame(&packed).unwrap();
    let packed2 = repack_frame(&values2, trailer2).unwrap();
    let (values3, trailer3) = unpack_frame(&packed2).unwrap();
    assert_eq!(values2, values3);
    assert_eq!(trailer2, trailer3);
});
