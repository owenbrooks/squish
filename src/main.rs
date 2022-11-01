use std::{
    env, fs,
    io::{self, BufRead, BufReader},
};
mod yuv4mpeg2;

fn main() {
    // Accept input either from stdin, or a filepath as first argument
    let input = env::args().nth(1);
    let mut reader: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).unwrap())),
    };

    let decoder = yuv4mpeg2::Decoder::new(&mut reader);
    let mut reader = decoder.read_header();

    let mut frame_buf = vec![0; reader.header.frame_bytes_length()];

    // Read through all frames
    let mut frame_count = 0;
    loop {
        match reader.next_frame(&mut frame_buf) {
            Some(_) => {
                frame_count += 1;
            }
            None => {
                break;
            }
        }
    }
    dbg!(frame_count);
}
