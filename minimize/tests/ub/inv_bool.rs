include!("../helper/transmute.rs");

fn main() { unsafe {
    let _b = transmute::<u8, bool>(2);
} }
