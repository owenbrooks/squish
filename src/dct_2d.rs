use crate::dct_1d;
use crate::yuv4mpeg2::Frame;

type MacroBlock = [[u8; 8]; 8];

// Quantises in 8x8 blocks
pub fn quantise_frame(frame: Frame) -> Frame {
    let blocks_y = divide(&frame.data_y, frame.height, frame.width);
    let coeffs_y = blocks_y.iter().map(|block| transform(*block));
    let quantised_y = coeffs_y.map(|block| quantise(block));
    let dequantised_y = quantised_y.map(|block| dequantise(block));
    let untransformed = dequantised_y
        .map(|block| inverse_transform(block))
        .collect();
    let data_y = concatenate(untransformed, frame.height, frame.width);

    let new_frame = Frame {
        width: frame.width,
        height: frame.height,
        data_y,
        data_cb: frame.data_cb,
        data_cr: frame.data_cr,
        // data_cb: vec![0; frame.chroma_len()],
        // data_cr: vec![0; frame.chroma_len()],
        color_space: frame.color_space,
    };

    new_frame
}

// Joins macroblocks back into a single frame
// Removes zero-padding to the right and bottom
fn concatenate(blocks: Vec<MacroBlock>, height: usize, width: usize) -> Vec<u8> {
    let mut values = vec![0; height * width];

    let block_count_y = (height as f32 / 8.).ceil() as usize;
    let block_count_x = (width as f32 / 8.).ceil() as usize;

    for j in 0..block_count_y {
        for i in 0..block_count_x {
            let block = blocks[j * block_count_x + i];

            let start_x = i * 8;
            let start_y = j * 8;
            let end_x = usize::min(start_x + 8, width);
            let end_y = usize::min(start_y + 8, height);

            for row in start_y..end_y {
                values[row * width + start_x..row * width + end_x]
                    .copy_from_slice(&block[row - start_y][0..end_x - start_x]);
            }
        }
    }

    values
}

const QUANT_MATRIX_50: [[f64; 8]; 8] = [
    [16., 11., 10., 16., 24., 40., 51., 61.],
    [12., 12., 14., 19., 26., 58., 60., 55.],
    [14., 13., 16., 24., 40., 57., 69., 56.],
    [14., 17., 22., 29., 51., 87., 80., 62.],
    [18., 22., 37., 56., 68., 109., 103., 77.],
    [24., 35., 55., 64., 81., 104., 113., 92.],
    [49., 64., 78., 87., 103., 121., 120., 101.],
    [72., 92., 95., 98., 112., 100., 103., 99.],
];
fn quantise(block: [[f64; 8]; 8]) -> [[f64; 8]; 8] {
    let mut output_block = block.clone();
    for i in 0..8 {
        for j in 0..8 {
            output_block[j][i] = (output_block[j][i]/(QUANT_MATRIX_50[j][i]*5.)).round();
        }
    }
    output_block
}
fn dequantise(block: [[f64; 8]; 8]) -> [[f64; 8]; 8] {
    let mut output_block = block.clone();
    for i in 0..8 {
        for j in 0..8 {
            output_block[j][i] = output_block[j][i]*QUANT_MATRIX_50[j][i]*5.;
        }
    }
    output_block
}

// Splits image data into square macroblocks of size 8x8, adding zero-padding
// where the block lies past the edge of the image to the right and/or bottom
fn divide(values: &[u8], height: usize, width: usize) -> Vec<MacroBlock> {
    let block_count_y = (height as f32 / 8.).ceil() as usize;
    let block_count_x = (width as f32 / 8.).ceil() as usize;

    let mut blocks = Vec::with_capacity(block_count_y * block_count_x);

    for j in 0..block_count_y {
        for i in 0..block_count_x {
            let mut block = [[0; 8]; 8];

            let start_x = i * 8;
            let start_y = j * 8;
            let end_x = usize::min(start_x + 8, width);
            let end_y = usize::min(start_y + 8, height);

            for row in start_y..end_y {
                block[row - start_y][0..end_x - start_x]
                    .copy_from_slice(&values[row * width + start_x..row * width + end_x]);
            }
            blocks.push(block);
        }
    }

    blocks
}

#[test]
fn does_divide_into_macroblocks() {
    const HEIGHT: usize = 23;
    const WIDTH: usize = 31;
    let data_y = [1; HEIGHT * WIDTH];

    let blocks = divide(&data_y, HEIGHT, WIDTH);
    assert_eq!(blocks.len(), 12); // check number of blocks
    assert_eq!(blocks[11][0][0], 1); // check values are copied over
    assert_eq!(blocks[11][7][7], 0); // check 0 padding
}

// shifts block values from 0,255 to -128,127
fn shift_and_normalise(block: MacroBlock) -> [[f64; 8]; 8] {
    let mut new_block = [[0.; 8]; 8];

    for i in 0..8 {
        for j in 0..8 {
            new_block[i][j] = (block[i][j] as i16 - 128) as f64;
        }
    }
    new_block
}

// maps values from -128,127 to 0,255
fn unshift_and_denormalise(block: [[f64; 8]; 8]) -> MacroBlock {
    let mut new_block = [[0; 8]; 8];

    for i in 0..8 {
        for j in 0..8 {
            // clamps to valid u8 (between 0 and 255)
            new_block[i][j] = ((block[i][j] as i16) + 128).max(0).min(255) as u8;
        }
    }
    new_block
}

// Performs transform as shown at https://en.wikipedia.org/wiki/Discrete_cosine_transform#M-D_DCT-II
fn transform(block: MacroBlock) -> [[f64; 8]; 8] {
    // Perform DCT along rows
    let mut shifted_block = shift_and_normalise(block);
    for k1 in 0..8 {
        dct_1d::transform(&mut shifted_block[k1]);
    }
    // Perform DCT along columns
    for i in 0..8 {
        let mut column = [0.; 8];
        for j in 0..8 {
            column[j] = shifted_block[j][i];
        }

        dct_1d::transform(&mut column);

        for j in 0..8 {
            shifted_block[j][i] = column[j];
        }
    }
    shifted_block
}
fn inverse_transform(coefficients: [[f64; 8]; 8]) -> MacroBlock {
    let mut intermediate_coeffs = coefficients.clone();

    // Perform DCT along rows
    for k1 in 0..8 {
        dct_1d::inverse_transform(&mut intermediate_coeffs[k1]);
    }
    // Perform DCT along columns
    for i in 0..8 {
        let mut column = [0.; 8];
        for j in 0..8 {
            column[j] = intermediate_coeffs[j][i];
        }

        dct_1d::inverse_transform(&mut column);

        for j in 0..8 {
            intermediate_coeffs[j][i] = column[j];
        }
    }

    let block = unshift_and_denormalise(intermediate_coeffs);
    block
}

#[test]
fn transforms_correctly() {
    let test_block: MacroBlock = [
        [52, 55, 61, 66, 70, 61, 64, 73],
        [63, 59, 55, 90, 109, 85, 69, 72],
        [62, 59, 68, 113, 144, 104, 66, 73],
        [63, 58, 71, 122, 154, 106, 70, 69],
        [67, 61, 68, 104, 126, 88, 68, 70],
        [79, 65, 60, 70, 77, 68, 58, 75],
        [85, 71, 64, 59, 55, 61, 65, 83],
        [87, 79, 69, 68, 65, 76, 78, 94],
    ];
    let transformed = transform(test_block);
    dbg!(transformed);
    let quantised = quantise(transformed);
    dbg!(quantised);
    let dequantised = dequantise(quantised);
    dbg!(dequantised);
    let inv = inverse_transform(dequantised);
    dbg!(inv);
    assert_eq!(inv[0][1], test_block[0][0]);
}
