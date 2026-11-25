#![no_main]

use libfuzzer_sys::fuzz_target;
use omnicalc::units::UnitRegistry;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    let val = f64::from_le_bytes(data[..8].try_into().unwrap());
    let rest = String::from_utf8_lossy(&data[8..]);
    let mut parts = rest.splitn(2, '\0');
    let from = parts.next().unwrap_or("");
    let to = parts.next().unwrap_or("");

    let reg = UnitRegistry::new_offline();
    let _ = reg.lookup(from);
    let _ = reg.lookup(to);
    let _ = reg.convert(val, from, to);
});
