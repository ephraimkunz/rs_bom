#![no_main]
use libfuzzer_sys::fuzz_target;
use rs_bom::RangeCollection;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(mut r) = RangeCollection::new(s) {
            r.canonicalize();
            let s = r.to_string();
        }
    }
});
