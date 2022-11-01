use std::{
    env, fs,
    io::{self, BufRead, BufReader},
};

use anyhow::Context;
mod yuv4mpeg2;

fn main() -> Result<(), anyhow::Error>{
    // Accept input either from stdin, or a filepath as first argument
    let input = env::args().nth(1);
    let mut reader: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).unwrap())),
    };

    let decoder = yuv4mpeg2::Decoder::new(&mut reader);
    let mut reader = decoder.read_header().context("Unable to read header")?;

    let mut frame_buf = vec![0; reader.header.frame_bytes_length()];

    let mut frame_count = 0;
    // Read through all frames
    while reader.next_frame(&mut frame_buf).is_some() {
        frame_count += 1;
    }
    dbg!(frame_count);

    Ok(())
}
