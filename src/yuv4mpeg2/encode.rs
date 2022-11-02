use std::{
    io::{BufWriter, Write},
};

use super::{Error, Frame, Header};

pub struct Writer<W: Write> {
    pub header: Header,
    sink: BufWriter<W>,
}

pub struct Encoder<W: Write> {
    sink: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn write_header(mut self, header: &Header) -> Result<Writer<W>, Error> {
        let header_string = header.to_string();
        self.sink.write(header_string.as_bytes())?;
        Ok(Writer {
            header: header.clone(),
            sink: self.sink,
        })
    }

    pub(crate) fn new(writer: W) -> Self {
        Encoder {
            sink: BufWriter::new(writer),
        }
    }
}

impl<W: Write> Writer<W> {
    pub fn write_frame(&mut self, frame: Frame) -> Result<(), Error> {
        let frame_marker_string = "FRAME\n";
        self.sink
            .write(frame_marker_string.as_bytes())?;

        let buf = frame.to_vec();
        self.sink
            .write(&buf)?;
        Ok(())
    }
}

impl ToString for Header {
    // Parses a yuv4mpeg2 header of the form described at https://wiki.multimedia.cx/index.php/YUV4MPEG2
    // into an instance of 'Header'
    fn to_string(&self) -> String {
        let header_string = format!(
            "YUV4MPEG2 W{width} H{height} F{num}:{den} {inter_mode_string} \
            {aspect_string} {color_string}\n",
            width = self.width,
            height = self.height,
            num = self.frame_rate_numerator,
            den = self.frame_rate_denominator,
            inter_mode_string = self.interlace_mode,
            aspect_string = self.pixel_aspect_ratio,
            color_string = self.color_space,
        );

        header_string
    }
}
