# RSMPEG API Reference for AI Context

## Overview
rsmpeg is a Rust wrapper for FFmpeg providing safe multimedia processing. Key modules: `avcodec` (encode/decode), `avformat` (containers), `avutil` (utilities), `avfilter` (filtering), `swresample` (audio resampling), `swscale` (video scaling), `error` (error handling), `ffi` (C bindings).

## Core Types

### AVFrame (Raw audio/video data)
```rust
let mut frame = AVFrame::new();
frame.set_format(ffi::AV_PIX_FMT_YUV420P);
frame.set_width(width);
frame.set_height(height);
frame.alloc_buffer()?;
frame.make_writable()?;
```
**Key methods:** `alloc_buffer()`, `make_writable()`, `image_copy_to_buffer()`, `hwframe_transfer_data()`, `get_buffer()`, `set_pts()`, `set_nb_samples()`

### AVPacket (Compressed data)
```rust
let mut packet = AVPacket::new();
packet.set_stream_index(stream_index);
packet.rescale_ts(input_timebase, output_timebase);
```

### AVCodecContext (Codec configuration)
```rust
let mut ctx = AVCodecContext::new(&codec);
ctx.set_width(320);
ctx.set_pix_fmt(ffi::AV_PIX_FMT_YUV420P);
ctx.apply_codecpar(&codecpar)?;
ctx.open(None)?;
```
**Key methods:** Encode/decode: `send_packet()`/`receive_frame()`, `send_frame()`/`receive_packet()`

## Essential Patterns

### Decoding Flow
```rust
let mut input_ctx = AVFormatContextInput::open(&path)?;
let (stream_idx, decoder) = input_ctx.find_best_stream(AVMEDIA_TYPE_VIDEO)?.unwrap();
let mut decode_ctx = AVCodecContext::new(&decoder);
decode_ctx.apply_codecpar(&input_ctx.streams()[stream_idx].codecpar())?;
decode_ctx.open(None)?;

while let Some(packet) = input_ctx.read_packet()? {
    decode_ctx.send_packet(Some(&packet))?;
    while let Ok(frame) = decode_ctx.receive_frame() {
        // Process frame
    }
}
decode_ctx.send_packet(None)?; // Flush
```

### Encoding Flow
```rust
let encoder = AVCodec::find_encoder(AV_CODEC_ID_H264)?;
let mut encode_ctx = AVCodecContext::new(&encoder);
encode_ctx.set_width(WIDTH);
encode_ctx.set_pix_fmt(AV_PIX_FMT_YUV420P);
encode_ctx.open(None)?;

for i in 0..num_frames {
    frame.set_pts(i);
    encode_ctx.send_frame(Some(&frame))?;
    while let Ok(packet) = encode_ctx.receive_packet() {
        // Write packet
    }
}
encode_ctx.send_frame(None)?; // Flush
```

### Filter Graph
```rust
let graph = AVFilterGraph::new();
let abuffer = AVFilter::get_by_name(c"abuffer")?;
let mut src = graph.alloc_filter_context(&abuffer, c"src")?;
src.opt_set(c"sample_rate", &sample_rate.to_string())?;
src.init_str(None)?;

// Link filters and configure
graph.config()?;

// Process
src.buffersrc_add_frame(Some(frame), None)?;
while let Ok(filtered_frame) = sink.buffersink_get_frame(None) {
    // Process filtered frame
}
```

### Audio Resampling
```rust
let mut swr = SwrContext::new(&dst_ch_layout, dst_sample_fmt, dst_rate,
                             &src_ch_layout, src_sample_fmt, src_rate)?;
swr.init()?;
let samples_out = swr.convert(dst_data.as_mut_ptr(), dst_nb_samples,
                             src_data.as_ptr(), src_nb_samples)?;
```

### Video Scaling
```rust
let mut sws = SwsContext::get_context(src_w, src_h, src_pix_fmt,
                                     dst_w, dst_h, dst_pix_fmt,
                                     ffi::SWS_BILINEAR, None, None, None)?;
sws.scale_frame(&src_frame, 0, src_height, &mut dst_frame)?;
```

## Error Handling
```rust
match decode_ctx.receive_frame() {
    Ok(frame) => { /* process */ },
    Err(RsmpegError::DecoderDrainError) | Err(RsmpegError::DecoderFlushedError) => break,
    Err(e) => return Err(e.into()),
}
```
**Key error types:** `DecoderDrainError`, `EncoderDrainError`, `BufferSinkDrainError`, `BufferSinkEofError`

## Constants Reference

### Media Types
- `AVMEDIA_TYPE_VIDEO`, `AVMEDIA_TYPE_AUDIO`, `AVMEDIA_TYPE_SUBTITLE`

### Pixel Formats
- `AV_PIX_FMT_YUV420P`, `AV_PIX_FMT_RGB24`, `AV_PIX_FMT_VAAPI`, `AV_PIX_FMT_CUDA`, `AV_PIX_FMT_NV12`

### Sample Formats
- `AV_SAMPLE_FMT_S16`, `AV_SAMPLE_FMT_FLT`, `AV_SAMPLE_FMT_FLTP`

### Channel Layouts
- `AV_CHANNEL_LAYOUT_MONO`, `AV_CHANNEL_LAYOUT_STEREO`, `AV_CHANNEL_LAYOUT_5POINT0`

### Codec IDs
- `AV_CODEC_ID_H264`, `AV_CODEC_ID_MPEG4`, `AV_CODEC_ID_AAC`

## Advanced Operations

### Hardware Acceleration
```rust
let hw_device_ctx = AVHWDeviceContext::create(device_type, None, None, 0)?;
decode_ctx.set_hw_device_ctx(hw_device_ctx);

// Hardware frame context
let mut hw_frames_ref = hw_device_ctx.hwframe_ctx_alloc();
hw_frames_ref.data().format = hw_format;
hw_frames_ref.init()?;
avctx.set_hw_frames_ctx(hw_frames_ref);

// Transfer data
sw_frame.hwframe_transfer_data(&hw_frame)?;
```

### Container Operations
```rust
// Input
let mut input_ctx = AVFormatContextInput::open(&path)?;
input_ctx.dump(0, &path)?;

// Output  
let mut output_ctx = AVFormatContextOutput::create(&path)?;
let mut stream = output_ctx.new_stream();
stream.set_codecpar(codecpar.clone());
output_ctx.write_header(&mut None)?;
output_ctx.write_trailer()?;
```

### Advanced Filter Operations
```rust
// Complex filter graphs
let outputs = AVFilterInOut::new(c"in", &mut buffersrc_ctx, 0);
let inputs = AVFilterInOut::new(c"out", &mut buffersink_ctx, 0);
graph.parse_ptr(filters_descr, Some(inputs), Some(outputs))?;

// Filter configuration
buffersink_ctx.opt_set(c"sample_formats", c"s16")?;
buffersink_ctx.opt_set_array(c"samplerates", 0, Some(&[8000]), ffi::AV_OPT_TYPE_INT)?;

// Frame processing with flags
src.buffersrc_add_frame(Some(frame), Some(ffi::AV_BUFFERSRC_FLAG_KEEP_REF as i32))?;
```

### Audio FIFO and Channel Layouts
```rust
// Audio FIFO buffering
let mut fifo = AVAudioFifo::new(sample_fmt, channels, initial_size)?;
fifo.realloc(fifo.size() + frame_size);
unsafe { fifo.write(samples_buffer.audio_data.as_ptr(), frame_size) }?;

// Channel layout handling
let ch_layout = AVChannelLayout::from_nb_channels(channels).into_inner();
if decode_ctx.ch_layout.order == ffi::AV_CHANNEL_ORDER_UNSPEC {
    decode_ctx.set_ch_layout(AVChannelLayout::from_nb_channels(decode_ctx.ch_layout.nb_channels).into_inner());
}
```

### Dictionary Options
```rust
let opts = AVDictionary::new(c"flags2", c"+export_mvs", 0);
decode_ctx.open(Some(opts))?;
```

## Utility Functions
- **Sample formats:** `get_bytes_per_sample()`, `sample_fmt_is_planar()`, `get_sample_fmt_name()`
- **Images:** `AVImage::new()`, `AVImage::get_buffer_size()`
- **Time:** `ts2str()`, `ts2timestr()`, `av_rescale_rnd()`
- **Audio:** `AVSamples::new()` for buffer allocation

## Memory Management
```rust
// Safe buffer access
let data = unsafe { std::slice::from_raw_parts(frame.data[0], buffer_size) };

// Audio samples
let samples = AVSamples::new(channels, nb_samples, sample_fmt, align)?;
// Access via samples.audio_data[plane_index]
```

## Special Patterns

### Parser-Based Decoding (raw streams)
```rust
let mut parser = AVCodecParserContext::init(codec.id)?;
let mut packet = AVPacket::new();
let (got_packet, offset) = parser.parse_packet(&mut decode_ctx, &mut packet, &data)?;
```

### Motion Vector Extraction
```rust
if let Some(side_data) = frame.get_side_data(ffi::AV_FRAME_DATA_MOTION_VECTORS) {
    let raw_motion_vectors = unsafe { side_data.as_motion_vectors() };
    for &mv in raw_motion_vectors {
        println!("Motion: src({},{}) dst({},{}) motion({},{}) flags={:#x}",
            mv.src_x, mv.src_y, mv.dst_x, mv.dst_y, mv.motion_x, mv.motion_y, mv.flags);
    }
}
```

### Remuxing Pattern
```rust
// Map input streams to output streams
let stream_mapping: Vec<Option<usize>> = input_streams.iter().enumerate().map(|(i, stream)| {
    let codec_type = stream.codecpar().codec_type();
    if codec_type.is_video() || codec_type.is_audio() || codec_type.is_subtitle() {
        let mut out_stream = output_ctx.new_stream();
        out_stream.set_codecpar(stream.codecpar().clone());
        Some(output_stream_index)
    } else { None }
}).collect();

// Process packets with timestamp rescaling
while let Some(mut packet) = input_ctx.read_packet()? {
    if let Some(out_idx) = stream_mapping[packet.stream_index as usize] {
        packet.rescale_ts(input_stream.time_base, output_stream.time_base);
        packet.set_stream_index(out_idx as i32);
        output_ctx.interleaved_write_frame(&mut packet)?;
    }
}
```

### Advanced Codec Setup
```rust
// Timing setup
decode_ctx.set_pkt_timebase(stream.time_base);
if let Some(framerate) = input_stream.guess_framerate() {
    decode_ctx.set_framerate(framerate);
}

// Global header for certain formats
if ofmt_ctx.oformat().flags & ffi::AVFMT_GLOBALHEADER as i32 != 0 {
    enc_ctx.set_flags(enc_ctx.flags | ffi::AV_CODEC_FLAG_GLOBAL_HEADER as i32);
}
```