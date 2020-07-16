#![no_main]
use libfuzzer_sys::fuzz_target;
use rs_bom::ReferenceCollection;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(mut r) = ReferenceCollection::new(s) {
            r.canonicalize();
            let s = r.to_string();
        }
    }
});
