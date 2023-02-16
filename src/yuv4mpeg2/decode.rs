use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use super::{ColorSpace, Error, Frame, Header, InterlaceMode, PixelAspectRatio};

pub struct Y4MReader<R: Read> {
    pub header: Header,
    source: BufReader<R>,
}

pub struct Decoder<R: Read> {
    source: BufReader<R>,
}

impl<R: Read> Decoder<R> {
    pub fn read_header(mut self) -> Result<Y4MReader<R>, Error> {
        let mut header_buf = String::new();
        self.source.read_line(&mut header_buf)?;
        let header = Header::from_str(&header_buf)?;
        Ok(Y4MReader {
            header,
            source: self.source,
        })
    }

    pub fn new(reader: R) -> Self {
        Decoder {
            source: BufReader::new(reader),
        }
    }
}

pub struct FrameIterator<R: Read> {
    reader: Y4MReader<R>,
}

impl<I: Read> Iterator for FrameIterator<I> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        let next_frame = self.reader.next_frame();
        match next_frame {
            Ok(frame) => frame,
            Err(_) => None,
        }
    }
}

impl<R: Read> IntoIterator for Y4MReader<R> {
    type Item = Frame;

    type IntoIter = FrameIterator<R>;

    fn into_iter(self) -> Self::IntoIter {
        FrameIterator {
            reader: self,
        }
    }

}

impl<R: Read> Y4MReader<R> {
    // todo: make this an iterator
    pub fn next_frame(&mut self) -> Result<Option<Frame>, Error> {
        let mut frame_buf = String::new();
        self.source.read_line(&mut frame_buf)?;

        if frame_buf.is_empty() {
            // end of file
            Ok(None)
        } else {
            let mut buf = vec![0; self.header.frame_bytes_length()];
            self.source.read_exact(&mut buf)?;
            let frame = Frame::from_buf(
                &buf,
                self.header.width,
                self.header.height,
                self.header.color_space,
            );
            Ok(Some(frame))
        }
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
        let first_token = parameter_strings.next().ok_or(Error::DecodeHeader)?;
        if first_token.to_lowercase() != "yuv4mpeg2" {
            return Err(Error::DecodeHeader);
        }

        for parameter_string in parameter_strings {
            match parameter_string.chars().nth(0) {
                Some(first_char) => {
                    match first_char {
                        'W' => {
                            let width = parameter_string[1..]
                                .parse::<usize>()
                                .map_err(|_| Error::DecodeDimensions)?;
                            header.width = width;
                        }
                        'H' => {
                            let height = parameter_string[1..]
                                .parse::<usize>()
                                .map_err(|_| Error::DecodeDimensions)?;
                            header.height = height;
                        }
                        'F' => {
                            parameter_string.split_once(':');
                            let (numerator, denominator) = parameter_string[1..]
                                .split_once(':')
                                .expect("Unable to parse frame rate");
                            header.frame_rate_numerator =
                                numerator.parse().map_err(|_| Error::DecodeFrameRate)?;
                            header.frame_rate_denominator =
                                denominator.parse().map_err(|_| Error::DecodeFrameRate)?;
                        }
                        'I' => match parameter_string {
                            "Ip" => header.interlace_mode = InterlaceMode::Ip,
                            "It" => header.interlace_mode = InterlaceMode::It,
                            "Ib" => header.interlace_mode = InterlaceMode::Ib,
                            "Im" => header.interlace_mode = InterlaceMode::Im,
                            _ => {
                                return Err(Error::DecodeInterlaceMode);
                            }
                        },
                        'A' => match parameter_string {
                            "A0:0" => header.pixel_aspect_ratio = PixelAspectRatio::Unknown,
                            "A1:1" => header.pixel_aspect_ratio = PixelAspectRatio::Square,
                            "A4:3" => header.pixel_aspect_ratio = PixelAspectRatio::NtscSvcd,
                            "A4:5" => header.pixel_aspect_ratio = PixelAspectRatio::NtscDvdNarrow,
                            "A32:27" => header.pixel_aspect_ratio = PixelAspectRatio::NtscDvdWide,
                            _ => header.pixel_aspect_ratio = PixelAspectRatio::Unknown,
                        },
                        'C' => match parameter_string {
                            "C420jpeg" => header.color_space = ColorSpace::C420jpeg,
                            "C420paldv" => header.color_space = ColorSpace::C420paldv,
                            "C420" => header.color_space = ColorSpace::C420,
                            "C422" => header.color_space = ColorSpace::C422,
                            "C444" => header.color_space = ColorSpace::C444,
                            "Cmono" => header.color_space = ColorSpace::Cmono,
                            "C420mpeg2" => header.color_space = ColorSpace::C420mpeg2,
                            _ => {
                                return Err(Error::DecodeColorSpace);
                            }
                        },
                        'X' => {} // ignore comments
                        _ => {}   // unknown, ignore
                    }
                }
                None => {
                    return Err(Error::DecodeHeader);
                }
            }
        }

        Ok(header)
    }
}
