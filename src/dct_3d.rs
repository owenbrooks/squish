use crate::{yuv4mpeg2::Frame, dct_1d};

// Accepts length eight vector of frames
// For each of the Y, Cb, and Cr components,
// and for each 2D pixel position in the image,
// it performs a 1D DCT along the frames i.e. in the time dimension
// Quantises and dequantises the resulting coefficients
// Performs the inverse transform
// Returns vector containing the same frames post-quantisation
pub fn quantise_chunk(chunk: Vec<Frame>, quantisation_factor: f64) -> Vec<Frame> {
    let mut quantised_chunk = chunk.clone();

    // Loop through pixel coordinates, perform 1D dct along frames at each coordinate

    // Y component
    for pixel_index in 0..chunk[0].data_y.len() {
        let mut temporal_vector = [0.; 8];
        for i in 0..8 {
            temporal_vector[i] = shift(chunk[i].data_y[pixel_index]);
        }
        dct_1d::transform(&mut temporal_vector);
        quantise(&mut temporal_vector, quantisation_factor);
        dequantise(&mut temporal_vector, quantisation_factor);
        dct_1d::inverse_transform(&mut temporal_vector);
        for i in 0..8 {
            quantised_chunk[i].data_y[pixel_index] = unshift(temporal_vector[i]);
        }
    }

    // Cb component
    for pixel_index in 0..chunk[0].data_cb.len() {
        let mut temporal_vector = [0.; 8];
        for i in 0..8 {
            temporal_vector[i] = shift(chunk[i].data_cb[pixel_index]);
        }
        dct_1d::transform(&mut temporal_vector);
        quantise(&mut temporal_vector, quantisation_factor);
        dequantise(&mut temporal_vector, quantisation_factor);
        dct_1d::inverse_transform(&mut temporal_vector);
        for i in 0..8 {
            quantised_chunk[i].data_cb[pixel_index] = unshift(temporal_vector[i]);
        }
    }

    // Cr component
    for pixel_index in 0..chunk[0].data_cr.len() {
        let mut temporal_vector = [0.; 8];
        for i in 0..8 {
            temporal_vector[i] = shift(chunk[i].data_cr[pixel_index]);
        }
        dct_1d::transform(&mut temporal_vector);
        quantise(&mut temporal_vector, quantisation_factor);
        dequantise(&mut temporal_vector, quantisation_factor);
        dct_1d::inverse_transform(&mut temporal_vector);
        for i in 0..8 {
            quantised_chunk[i].data_cr[pixel_index] = unshift(temporal_vector[i]);
        }
    }

    quantised_chunk
}


// Divides each element by the quantisation factor and rounds the result to the
// nearest integer
fn quantise(vector: &mut [f64; 8], quantisation_factor: f64) {
    for elem in vector.iter_mut() {
        *elem = (*elem / quantisation_factor).round();
    }
}
// Multplies by the quantisation factor
fn dequantise(vector: &mut [f64; 8], quantisation_factor: f64) {
    for elem in vector.iter_mut() {
        *elem = *elem * quantisation_factor;
    }
}


// Shifts values from the range [0,255] to [-128.0,127.0]
fn shift(value: u8) -> f64 {
    (value as i16 - 128) as f64
}
// Shifts values from the range [-128.0,127.0] to [0,255]
fn unshift(value: f64) -> u8 {
    ((value as i16) + 128).max(0).min(255) as u8
}
