use std::{
    env, fs,
    io::{self, BufRead, BufReader, Write, BufWriter},
};

use anyhow::Context;
mod yuv4mpeg2;
mod dct_2d;

fn main() -> Result<(), anyhow::Error>{
    // Accept input either from stdin, or a filepath as first argument
    let input = env::args().nth(1);
    let mut reader: Box<dyn BufRead> = match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(filename) => Box::new(BufReader::new(fs::File::open(filename).context("Failed to open file. Check that it exists.")?)),
    };

    let decoder = yuv4mpeg2::Decoder::new(&mut reader);
    let mut reader = decoder.read_header().context("Failed to read header")?;

    // Output either to stdout, or a filepath as second argument if given
    let output = env::args().nth(2);
    let writer: Box<dyn Write> = match output {
        None => Box::new(BufWriter::new(io::stdout())), // TODO: fix writing to stdout as it doesn't seem to work
        Some(filename) => Box::new(BufWriter::new(fs::File::create(filename).context("Failed to create file. Check that the target directory exists.")?)),
    };
    let encoder = yuv4mpeg2::Encoder::new(writer);
    let mut writer = encoder.write_header(&reader.header).context("Failed to write header")?;

    // Read through all frames and write them out to a new file
    let mut frame_count = 0;
    while let Some(frame) = reader.next_frame().context("Failed to read frame")? {
        writer.write_frame(frame).context("Failed to write frame")?;
        frame_count += 1;

        // if frame_count >= 10 {
        //     break;
        // }
    }
    dbg!(frame_count);

    

    Ok(())
}
