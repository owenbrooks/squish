// Coding based on description here: https://wiki.multimedia.cx/index.php/YUV4MPEG2
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Decode error")]
    DecodeError(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

pub struct Reader<R: Read> {
    pub header: Header,
    source: BufReader<R>,
}

pub struct Decoder<R: Read> {
    source: BufReader<R>,
}

impl<R: Read> Decoder<R> {
    pub fn read_header(mut self) -> Result<Reader<R>, Error> {
        let mut header_buf = String::new();
        self.source.read_line(&mut header_buf)?;
        let header = Header::from_str(&header_buf)?;
        Ok(Reader {
            header,
            source: self.source,
        })
    }

    pub(crate) fn new(reader: R) -> Self {
        Decoder {
            source: BufReader::new(reader),
        }
    }
}

impl<R: Read> Reader<R> {
    pub fn next_frame(&mut self, buf: &mut [u8]) -> Option<()> {
        let mut frame_buf = String::new();
        self.source
            .read_line(&mut frame_buf)
            .expect("Unable to read line");

        if frame_buf.is_empty() {
            // end of file
            None
        } else {
            self.source.read_exact(buf).expect("Unable to read frame");
            Some(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Header {
    pub width: usize,
    pub height: usize,
    pub frame_rate_numerator: usize,
    pub frame_rate_denominator: usize,
    pub interlace_mode: InterlaceMode,
    pub pixel_aspect_ratio: PixelAspectRatio,
    pub color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy)]
pub enum InterlaceMode {
    Unknown,
    Ip,
    It,
    Ib,
    Im,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelAspectRatio {
    Unknown,
    Square,
    NtscSvcd,
    NtscDvdNarrow,
    NtscDvdWide,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorSpace {
    C420jpeg,  // 4:2:0 with biaxially-displaced chroma planes
    C420paldv, // 4:2:0 with vertically-displaced chroma planes
    C420,      // 4:2:0 with coincident chroma planes
    C422,      // 4:2:2
    C444,      // 4:4:4
    Cmono,     // YCbCr plane only
    C420mpeg2, // TODO: investigate
}

impl Default for Header {
    fn default() -> Self {
        Header {
            width: 0,
            height: 0,
            frame_rate_numerator: 0,
            frame_rate_denominator: 0,
            interlace_mode: InterlaceMode::Unknown,
            pixel_aspect_ratio: PixelAspectRatio::Unknown,
            color_space: ColorSpace::C420,
        }
    }
}

impl Header {
    pub fn frame_bytes_length(&self) -> usize {
        let colorspace_multiplier = match self.color_space {
            ColorSpace::C420
            | ColorSpace::C420jpeg
            | ColorSpace::C420mpeg2
            | ColorSpace::C420paldv => 3. / 2.,
            ColorSpace::C422 => 2.,
            ColorSpace::C444 => 3.,
            ColorSpace::Cmono => 1., // todo: check
        };
        (self.width as f32 * self.height as f32 * colorspace_multiplier) as usize
    }
}

impl FromStr for Header {
    type Err = Error;

    // Parses a yuv4mpeg2 header of the form described at https://wiki.multimedia.cx/index.php/YUV4MPEG2
    // into an instance of 'Header'
    fn from_str(header_string: &str) -> Result<Self, Error> {
        let header_string = header_string.trim_end(); // removes trailing newline character
        let mut parameter_strings = header_string.split(' ');

        let mut header = Header::default();

        // Check if first word is YUV4MPEG2
        let first_token = parameter_strings.next().ok_or(Error::DecodeError(
            "Doesn't appear to be a YUV4MPEG2 file".to_string(),
        ))?;
        if first_token.to_lowercase() != "yuv4mpeg2" {
            return Err(Error::DecodeError(
                "Doesn't appear to be a YUV4MPEG2 file".to_string(),
            ));
        }

        for parameter_string in parameter_strings {
            match parameter_string.chars().nth(0) {
                Some(first_char) => {
                    match first_char {
                        'W' => {
                            let width = parameter_string[1..].parse::<usize>().map_err(|_| {
                                Error::DecodeError("Unable to parse width".to_string())
                            })?;
                            header.width = width;
                        }
                        'H' => {
                            let height = parameter_string[1..]
                                .parse::<usize>()
                                .expect("Unable to parse height");
                            header.height = height;
                        }
                        'F' => {
                            parameter_string.split_once(':');
                            let (numerator, denominator) = parameter_string[1..]
                                .split_once(':')
                                .expect("Unable to parse frame rate");
                            header.frame_rate_numerator = numerator.parse().unwrap();
                            header.frame_rate_denominator = denominator.parse().unwrap();
                        }
                        'I' => match parameter_string {
                            "Ip" => header.interlace_mode = InterlaceMode::Ip,
                            "It" => header.interlace_mode = InterlaceMode::It,
                            "Ib" => header.interlace_mode = InterlaceMode::Ib,
                            "Im" => header.interlace_mode = InterlaceMode::Im,
                            _ => {
                                return Err(Error::DecodeError("Unknown interlace mode provided".to_string()));
                            }
                        }
                        'A' => match parameter_string {
                            "A0:0" => header.pixel_aspect_ratio = PixelAspectRatio::Unknown,
                            "A1:1" => header.pixel_aspect_ratio = PixelAspectRatio::Square,
                            "A4:3" => header.pixel_aspect_ratio = PixelAspectRatio::NtscSvcd,
                            "A4:5" => header.pixel_aspect_ratio = PixelAspectRatio::NtscDvdNarrow,
                            "A32:27" => header.pixel_aspect_ratio = PixelAspectRatio::NtscDvdWide,
                            _ => header.pixel_aspect_ratio = PixelAspectRatio::Unknown,
                        }
                        'C' => match parameter_string {
                            "C420jpeg" => header.color_space = ColorSpace::C420jpeg,
                            "C420paldv" => header.color_space = ColorSpace::C420paldv,
                            "C420" => header.color_space = ColorSpace::C420,
                            "C422" => header.color_space = ColorSpace::C422,
                            "C444" => header.color_space = ColorSpace::C444,
                            "Cmono" => header.color_space = ColorSpace::Cmono,
                            "C420mpeg2" => header.color_space = ColorSpace::C420mpeg2,
                            _ => {
                                return Err(Error::DecodeError("Color space invalid".to_string()));
                            }
                        }
                        'X' => {} // ignore comments
                        _ => {}   // unknown, ignore
                    }
                }
                None => {
                    return Err(Error::DecodeError("Empty token received".to_string()));
                }
            }
        }

        Ok(header)
    }
}
