use std::io::{BufWriter, Write};

use super::{ColorSpace, Error, Frame, Header, InterlaceMode, PixelAspectRatio};

pub struct Y4MWriter<W: Write> {
    pub header: Header,
    sink: BufWriter<W>,
}

pub struct Encoder<W: Write> {
    sink: BufWriter<W>,
}

impl<W: Write> Encoder<W> {
    pub fn write_header(mut self, header: &Header) -> Result<Y4MWriter<W>, Error> {
        let header_string = header.to_string();
        self.sink.write(header_string.as_bytes())?;
        Ok(Y4MWriter {
            header: header.clone(),
            sink: self.sink,
        })
    }

    pub fn new(writer: W) -> Self {
        Encoder {
            sink: BufWriter::new(writer),
        }
    }
}

impl<W: Write> Y4MWriter<W> {
    pub fn write_frame(&mut self, frame: Frame) -> Result<(), Error> {
        let frame_marker_string = "FRAME\n";
        self.sink.write(frame_marker_string.as_bytes())?;

        let buf = frame.to_vec();
        self.sink.write(&buf)?;
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

impl std::fmt::Display for InterlaceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InterlaceMode::Unknown => write!(f, "I?"),
            InterlaceMode::Ip => write!(f, "Ip"),
            InterlaceMode::It => write!(f, "It"),
            InterlaceMode::Ib => write!(f, "Ib"),
            InterlaceMode::Im => write!(f, "Im"),
        }
    }
}

impl std::fmt::Display for PixelAspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PixelAspectRatio::Unknown => write!(f, "A0:0"),
            PixelAspectRatio::Square => write!(f, "A1:1"),
            PixelAspectRatio::NtscSvcd => write!(f, "A4:3"),
            PixelAspectRatio::NtscDvdNarrow => write!(f, "A4:5"),
            PixelAspectRatio::NtscDvdWide => write!(f, "A32:27"),
        }
    }
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorSpace::C420jpeg => write!(f, "C420jpeg"),
            ColorSpace::C420paldv => write!(f, "C420paldv"),
            ColorSpace::C420 => write!(f, "C420"),
            ColorSpace::C422 => write!(f, "C422"),
            ColorSpace::C444 => write!(f, "C444"),
            ColorSpace::Cmono => write!(f, "Cmono"),
            ColorSpace::C420mpeg2 => write!(f, "C420mpeg2"),
        }
    }
}
