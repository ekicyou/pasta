use crate::error::*;
use std::{ffi::*, path::*};

pub(crate) trait Shiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool>;
    fn request<S: AsRef<str>>(&mut self, request: S) -> MyResult<String>;
}

#[derive(Default)]
pub(crate) struct PastaShiori {
    hinst: isize,
    load_dir: Option<PathBuf>,
}

impl Drop for PastaShiori {
    fn drop(&mut self) {}
}

impl Shiori for PastaShiori {
    fn load<S: AsRef<OsStr>>(&mut self, hinst: isize, load_dir: S) -> MyResult<bool> {
        self.hinst = hinst;
        self.load_dir = Some(load_dir.as_ref().into());
        Ok(true)
    }

    fn request<S: AsRef<str>>(&mut self, req: S) -> MyResult<String> {
        let req = req.as_ref();
        Ok(format!("PastaShiori received request: {}", req))
    }
}
