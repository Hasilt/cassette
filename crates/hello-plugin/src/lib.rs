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

#[no_mangle]
pub extern "C" fn greet_name(name_ptr: i32, name_len: i32, out_ptr: i32, out_max: i32) -> i32 {
    let src = name_ptr as usize;
    let len = name_len as usize;
    let dst = out_ptr as usize;
    let cap = out_max as usize;

    let name = unsafe { core::slice::from_raw_parts(src as *const u8, len) };
    let buf = unsafe { core::slice::from_raw_parts_mut(dst as *mut u8, cap) };

    let mut i = 0;
    let push = |buf: &mut [u8], i: &mut usize, b: u8| {
        if *i < buf.len() {
            buf[*i] = b;
            *i += 1;
        }
    };

    for &b in b"Hello, " {
        push(buf, &mut i, b);
    }
    for &b in name {
        push(buf, &mut i, b);
    }
    push(buf, &mut i, b'!');

    i as i32
}