use crate::yuv4mpeg2::Frame;

type MacroBlock = [[u8; 8]; 8];

// Quantises in 8x8 blocks
pub fn quantise_frame(frame: Frame) -> Frame {
    let blocks_y = divide(&frame.data_y, frame.height, frame.width);
    let coeffs_y = blocks_y.iter().map(|block| transform(*block));
    let quantised_y = coeffs_y.map(|block| quantise(block));
    let dequantised_y = quantised_y.map(|block| inverse_transform(block)).collect();
    let data_y = concatenate(dequantised_y, frame.height, frame.width);

    let new_frame = Frame {
        width: frame.width,
        height: frame.height,
        data_y,
        data_cb: frame.data_cb,
        data_cr: frame.data_cr,
        color_space: frame.color_space,
    };

    new_frame
}

// Joins macroblocks back into a single frame
// Removes zero-padding to the right and bottom
fn concatenate(blocks: Vec<MacroBlock>, height: usize, width: usize) -> Vec<u8> {
    let mut values = vec![0; height*width];

    let block_count_y = (height as f32 / 8.).ceil() as usize;
    let block_count_x = (width as f32 / 8.).ceil() as usize;

    for j in 0..block_count_y {
        for i in 0..block_count_x {
            let block = blocks[j*block_count_x + i];

            let start_x = i*8;
            let start_y = j*8;
            let end_x = usize::min(start_x + 8, width);
            let end_y = usize::min(start_y + 8, height);

            for row in start_y..end_y {
                values[row*width+start_x..row*width+end_x].copy_from_slice(&block[row-start_y][0..end_x-start_x]);
            }
        }
    }

    values
}

fn quantise(block: MacroBlock) -> MacroBlock {
    // TODO: implement
    block
}

// Splits image data into square macroblocks of size 8x8, adding zero-padding
// where the block lies past the edge of the image to the right and/or bottom
fn divide(values: &[u8], height: usize, width: usize) -> Vec<MacroBlock> {
    let block_count_y = (height as f32 / 8.).ceil() as usize;
    let block_count_x = (width as f32 / 8.).ceil() as usize;

    let mut blocks = Vec::with_capacity(block_count_y*block_count_x);

    for j in 0..block_count_y {
        for i in 0..block_count_x {
            let mut block = [[0; 8]; 8];

            let start_x = i*8;
            let start_y = j*8;
            let end_x = usize::min(start_x + 8, width);
            let end_y = usize::min(start_y + 8, height);

            for row in start_y..end_y {
                block[row-start_y][0..end_x-start_x].copy_from_slice(&values[row*width+start_x..row*width+end_x]);

                // TODO: remove
                // for col in start_x..end_x {
                //     block[row-start_y][col-start_x] = values[row*width+col];
                // }
            }
            blocks.push(block);
        }
    }

    blocks
}

#[test]
fn divides_into_macroblocks() {
    const HEIGHT: usize = 23;
    const WIDTH: usize = 31;
    let data_y = [1; HEIGHT*WIDTH];

    let blocks = divide(&data_y, HEIGHT, WIDTH);
    assert_eq!(blocks.len(), 12); // check number of blocks
    assert_eq!(blocks[11][0][0], 1); // check values are copied over
    assert_eq!(blocks[11][7][7], 0); // check 0 padding
}

// Performs transform as shown at https://en.wikipedia.org/wiki/Discrete_cosine_transform#M-D_DCT-II
fn transform(block: MacroBlock) -> MacroBlock {
    let mut coefficients = [[0; 8]; 8];
    let height = 8;
    let width = 8;

    for k1 in 0..height {
        for k2 in 0..width {
            let mut sum = 0.0;
            for n1 in 0..height {
                for n2 in 0..width {
                    let value = block[n1][n2] as f32;
                    let row_wise = (std::f32::consts::PI * (n1 as f32 + 0.5) * (k1 as f32)
                        / (height as f32))
                        .cos();
                    let col_wise = (std::f32::consts::PI * (n2 as f32 + 0.5) * (k2 as f32)
                        / (width as f32))
                        .cos();
                    sum += value * row_wise * col_wise;
                }
            }

            coefficients[k1][k2] = sum as u8;
        }
    }

    coefficients
}

fn inverse_transform(coefficients: MacroBlock) -> MacroBlock {
    let mut block = [[0; 8]; 8];
    let height = 8;
    let width = 8;

    for k1 in 0..height {
        for k2 in 0..width {
            let mut sum = 0.0;
            for n1 in 0..height {
                for n2 in 0..width {
                    if n1 == 0 && n2 == 0 {
                        // skip for DC term
                        continue;
                    }
                    let value = coefficients[n1][n2] as f32;
                    let row_wise = (std::f32::consts::PI * (k1 as f32 + 0.5) * (n1 as f32)
                        / (height as f32))
                        .cos();
                    let col_wise = (std::f32::consts::PI * (k2 as f32 + 0.5) * (n2 as f32)
                        / (width as f32))
                        .cos();
                    sum += value * row_wise * col_wise;
                }
            }

            let dc_term = coefficients[0][0] as f32;
            block[k1][k2] = (sum + 1. / (2. * dc_term)) as u8;
        }
    }

    block
}
