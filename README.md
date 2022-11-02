# squish
Exploration of compression algorithms

Convert your video file to YUV4MPEG2 format:
`ffmpeg -i input.mp4 -f yuv4mpegpipe input.v`

Create compressed version:
`cargo run --release input.v output.v`