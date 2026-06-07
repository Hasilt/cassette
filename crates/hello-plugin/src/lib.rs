#![no_std]

const GREETING: &[u8] = b"Hello from Cassette!\0";

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn greet() -> *const u8 {
    GREETING.as_ptr()
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}