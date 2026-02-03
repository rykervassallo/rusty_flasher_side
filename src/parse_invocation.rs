use std::{env, io};
use std::path::PathBuf;

pub fn parse_invocation() -> io::Result<PathBuf> {
    let path: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: my-program <path>");

    Ok(path)
}
