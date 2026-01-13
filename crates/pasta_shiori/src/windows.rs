//! Windows SHIORI DLL interface
//!
//! Provides SHIORI protocol entry points for Windows DLL.

/// SHIORI load entry point
///
/// # Safety
/// This function is called from external C code.
#[unsafe(no_mangle)]
pub extern "C" fn load(_h: isize, _len: usize) -> bool {
    true
}

/// SHIORI unload entry point
///
/// # Safety
/// This function is called from external C code.
#[unsafe(no_mangle)]
pub extern "C" fn unload() -> bool {
    true
}

/// SHIORI request entry point
///
/// # Safety
/// This function is called from external C code.
#[unsafe(no_mangle)]
pub extern "C" fn request(_h: isize, _len: *mut usize) -> isize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_unload() {
        assert!(load(0, 0));
        assert!(unload());
    }
}
