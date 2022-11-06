use std::{
    fs,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;

use crate::dct_2d::quantise_frame;
mod dct_1d;
mod dct_2d;
mod yuv4mpeg2;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file (must be in YUV4MPEG2 format)
    #[arg(short, long)]
    input_file: PathBuf,

    /// Output file (will be in YUV4MPEG2 format)
    #[arg(short, long, default_value = "output.y4m")]
    output_file: PathBuf,

    /// Quantisation factor (higher results in lower quality)
    #[arg(short, long, default_value_t = 1.)]
    quantisation_factor: f64,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    // Accept input either from stdin, or a filepath as first argument
    let mut reader = BufReader::new(
        fs::File::open(args.input_file).context("Failed to open file. Check that it exists.")?,
    );

    let decoder = yuv4mpeg2::Decoder::new(&mut reader);
    let mut reader = decoder.read_header().context("Failed to read header")?;

    // Output either to stdout, or a filepath as second argument if given
    let writer = BufWriter::new(
        fs::File::create(args.output_file)
            .context("Failed to create file. Check that the target directory exists.")?,
    );
    let encoder = yuv4mpeg2::Encoder::new(writer);
    let mut writer = encoder
        .write_header(&reader.header)
        .context("Failed to write header")?;

    // Read through all frames and write them out to a new file
    let mut frame_count = 0;
    while let Some(frame) = reader.next_frame().context("Failed to read frame")? {
        let new_frame = quantise_frame(frame, args.quantisation_factor);
        // todo: figure out chroma height and widths
        // let coeffs_cb = dct_2d::transform(&frame.data_cb, frame.height, frame.width);
        // let coeffs_cr = dct_2d::transform(&frame.data_cr, frame.height, frame.width);
        writer
            .write_frame(new_frame)
            .context("Failed to write frame")?;
        frame_count += 1;

        // if frame_count >= 20 {
        //     break;
        // }
    }
    dbg!(frame_count);

    Ok(())
}
