// Coding based on description here: https://wiki.multimedia.cx/index.php/YUV4MPEG2

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
    pub fn read_header(mut self) -> Reader<R> {
        let mut header_buf = String::new();
        self.source
            .read_line(&mut header_buf)
            .expect("Buffer not large enough to read header or header malformed.");
        let header = Header::from_str(&header_buf).expect("Unable to parse header");
        Reader {
            header,
            source: self.source,
        }
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

        if frame_buf.len() == 0 {
            // end of file
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
    pub interlace_mode: Option<InterlaceMode>,
    pub pixel_aspect_ratio: Option<PixelAspectRatio>,
    pub color_space: ColorSpace,
    pub x_color_range: Option<XColorRange>,
}

#[derive(Debug, Clone, Copy)]
pub enum InterlaceMode {
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
    NtscDvd,
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

#[derive(Debug, Clone, Copy)]
pub enum XColorRange {
    Full,
    Limited,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            width: 0,
            height: 0,
            frame_rate_numerator: 0,
            frame_rate_denominator: 0,
            interlace_mode: None,
            pixel_aspect_ratio: None,
            color_space: ColorSpace::C420,
            x_color_range: None,
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
    type Err = std::num::ParseIntError;

    // Parses a yuv4mpeg2 header of the form described at https://wiki.multimedia.cx/index.php/YUV4MPEG2
    //  into an instance of 'Header'
    fn from_str(header_string: &str) -> Result<Self, Self::Err> {
        let header_string = header_string.trim_end(); // removes trailing newline character
        let parameter_strings = header_string.split(" ");

        let mut header = Header::default();

        // TODO: check if first "param" is YUV4MPEG2
        for parameter_string in parameter_strings {
            match parameter_string.chars().nth(0) {
                Some(first_char) => {
                    match first_char {
                        'Y' => {} // nothing
                        'W' => {
                            let width = parameter_string[1..]
                                .parse::<usize>()
                                .expect("Unable to parse width");
                            header.width = width;
                        }
                        'H' => {
                            let height = parameter_string[1..]
                                .parse::<usize>()
                                .expect("Unable to parse height");
                            header.height = height;
                        }
                        'F' => {
                            parameter_string.split_once(":");
                            let (numerator, denominator) = parameter_string[1..]
                                .split_once(":")
                                .expect("Unable to parse frame rate");
                            header.frame_rate_numerator = numerator.parse().unwrap();
                            header.frame_rate_denominator = denominator.parse().unwrap();
                        }
                        'I' => {} // TODO
                        'A' => {} // TODO
                        'C' => {
                            match parameter_string {
                                "C420jpeg" => header.color_space = ColorSpace::C420jpeg,
                                "C420paldv" => header.color_space = ColorSpace::C420paldv,
                                "C420" => header.color_space = ColorSpace::C420,
                                "C422" => header.color_space = ColorSpace::C422,
                                "C444" => header.color_space = ColorSpace::C444,
                                "Cmono" => header.color_space = ColorSpace::Cmono,
                                "C420mpeg2" => header.color_space = ColorSpace::C420mpeg2,
                                _ => {
                                    dbg!(parameter_string);
                                    panic!("Color space invalid")
                                } // invalid
                            }
                        } // TODO
                        'X' => {} // ignore comments
                        _ => {}   // invalid
                    }
                }
                None => {
                    // error
                }
            }
        }

        Ok(header)
    }
}
