#![no_main]
use libfuzzer_sys::fuzz_target;
use rs_bom::ReferenceCollection as rc;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = rc::new(s);
    }
});
