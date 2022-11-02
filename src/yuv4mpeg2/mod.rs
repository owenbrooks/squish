// Coding based on description here: https://wiki.multimedia.cx/index.php/YUV4MPEG2
mod decode;
mod encode;
pub use decode::Decoder;
pub use encode::Encoder;

pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub color_space: ColorSpace,
    pub data_y: Vec<u8>,
    pub data_cb: Vec<u8>,
    pub data_cr: Vec<u8>,
}
fn chroma_len_from_space(color_space: ColorSpace, width: usize, height: usize) -> usize {
    match color_space {
        ColorSpace::C444 => height * width,
        ColorSpace::C422 => height * width / 2,
        ColorSpace::C420 | ColorSpace::C420jpeg | ColorSpace::C420mpeg2 | ColorSpace::C420paldv => {
            height * width / 4
        }
        ColorSpace::Cmono => height * width, // TODO: check definition of Cmono
    }
}
impl Frame {
    pub fn chroma_len(&self) -> usize {
        chroma_len_from_space(self.color_space, self.width, self.height)
    }
    pub fn from_buf(buf: &[u8], width: usize, height: usize, color_space: ColorSpace) -> Self {
        let y_len = height * width;
        let chroma_len = chroma_len_from_space(color_space, width, height);

        let data_y = buf[..y_len].to_vec();
        let data_cb = buf[y_len..y_len + chroma_len].to_vec();
        let data_cr = buf[y_len + chroma_len..y_len + 2 * chroma_len].to_vec();

        Frame {
            width,
            height,
            color_space,
            data_y,
            data_cb,
            data_cr,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        [
            self.data_y.clone(),
            self.data_cb.clone(),
            self.data_cr.clone(),
        ]
        .concat()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse header")]
    DecodeHeader,
    #[error("Unable to parse dimensions")]
    DecodeDimensions,
    #[error("Unable to parse color space")]
    DecodeColorSpace,
    #[error("Unable to parse frame rate")]
    DecodeFrameRate,
    #[error("Unable to parse interlace mode")]
    DecodeInterlaceMode,

    #[error(transparent)]
    IOError(#[from] std::io::Error),
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
