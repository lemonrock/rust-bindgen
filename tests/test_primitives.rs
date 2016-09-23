use bindgen;
use support::assert_bind_eq;

#[test]
fn unsigned() {
    let bindings = bindgen::Builder::new("tests/headers/unsigned.h")
        .generate()
        .unwrap()
        .to_string();

    assert!(bindings.contains("pub type size_t = usize;"));
    assert!(bindings.contains("pub type uintptr_t = usize;"));
    assert!(bindings.contains("pub type uint8_t = u8;"));
    assert!(bindings.contains("pub type uint16_t = u16;"));
    assert!(bindings.contains("pub type uint32_t = u32;"));
    assert!(bindings.contains("pub type uint64_t = u64;"));
    assert!(bindings.contains("pub static mut c: wchar_t;"));
}

#[test]
fn signed() {
    let bindings = bindgen::Builder::new("tests/headers/signed.h")
        .generate()
        .unwrap()
        .to_string();

    assert!(bindings.contains("pub type ptrdiff_t = isize;"));
    assert!(bindings.contains("pub type intptr_t = isize;"));
    assert!(bindings.contains("pub type ssize_t = isize;"));
    assert!(bindings.contains("pub type int8_t = i8;"));
    assert!(bindings.contains("pub type int16_t = i16;"));
    assert!(bindings.contains("pub type int32_t = i32;"));
    assert!(bindings.contains("pub type int64_t = i64;"));
}

#[test]
fn floats() {
    assert_bind_eq(Default::default(),
                   "headers/floats.h",
                   "
    extern \"C\" {
        pub static mut f: f32;
        pub static mut d: f64;
    }
    ");
}

#[test]
fn vectors() {
    assert_bind_eq(Default::default(),
                   "headers/vectors.h",
                   "
    pub type __v4si = [::std::os::raw::c_int; 4usize];
    pub type __v4sf = [f32; 4usize];
    pub type __m128i = [::std::os::raw::c_int; 4usize];
    pub type __v4su = [::std::os::raw::c_uint; 4usize];
    ");
}

#[test]
fn i128() {
    assert_bind_eq(Default::default(),
                   "headers/i128.h",
                   "
    #[repr(C)]
    #[derive(Copy, Clone)]
    #[derive(Debug)]
    pub struct int128 {
        pub hi: ::std::os::raw::c_longlong,
        pub lo: ::std::os::raw::c_ulonglong,
    }
    impl ::std::default::Default for int128 {
        fn default() -> Self { unsafe { ::std::mem::zeroed() } }
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    #[derive(Debug)]
    pub struct uint128 {
        pub hi: ::std::os::raw::c_ulonglong,
        pub lo: ::std::os::raw::c_ulonglong,
    }
    impl ::std::default::Default for uint128 {
        fn default() -> Self { unsafe { ::std::mem::zeroed() } }
    }
    ");
}
