use std::{fs::File, io::Read};

const RANDOM_PATH: &str = "/dev/random";

pub fn fill_f32(slice: &mut [f32]) {}

fn get_random_file() -> File {
    File::open(RANDOM_PATH).unwrap_or_else(|err| {
        panic!(
            "Eeeh... your system doesn't seem to have `{}`? Please create an issue.\n{}",
            RANDOM_PATH, err
        )
    })
}
