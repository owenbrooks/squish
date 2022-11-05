# squish
Exploration of compression algorithms

Convert your video file to YUV4MPEG2 format:
`ffmpeg -i input.mp4 -f yuv4mpegpipe input.v`

Create compressed version:
`cargo run --release input.v output.v`

https://cs.stanford.edu/people/eroberts/courses/soco/projects/data-compression/lossy/jpeg/coeff.htm
https://en.wikipedia.org/wiki/JPEG
https://en.wikipedia.org/wiki/Discrete_cosine_transform#DCT-II

Flamegraph: `CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- input.v out.v`