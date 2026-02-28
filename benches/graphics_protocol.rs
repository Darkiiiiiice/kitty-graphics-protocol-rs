//! Benchmark tests for the Kitty graphics protocol library

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use kitty_graphics_protocol::{Action, Command, DeleteTarget, ImageFormat, Response};
use rand::Rng;

// =============================================================================
// Test Data Generation
// =============================================================================

/// Generate random RGBA image data
fn generate_rgba_data(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 4) as usize;
    let mut rng = rand::rng();
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.random());
    }
    data
}

/// Generate random RGB image data
fn generate_rgb_data(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 3) as usize;
    let mut rng = rand::rng();
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.random());
    }
    data
}

/// Generate a simple PNG-like header (not a valid PNG, just for testing)
fn generate_png_like_data(size: usize) -> Vec<u8> {
    let mut rng = rand::rng();
    let mut data = Vec::with_capacity(size);
    // PNG signature
    data.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    for _ in 8..size {
        data.push(rng.random());
    }
    data
}

// =============================================================================
// Command Builder Benchmarks
// =============================================================================

fn bench_command_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_builder");

    // Basic command construction
    group.bench_function("basic_transmit", |b| {
        b.iter(|| {
            Command::builder()
                .action(Action::Transmit)
                .format(ImageFormat::Png)
                .image_id(1)
                .build()
        })
    });

    // Complex command with all options
    group.bench_function("complex_command", |b| {
        b.iter(|| {
            Command::builder()
                .action(Action::TransmitAndDisplay)
                .format(ImageFormat::Rgba)
                .dimensions(1920, 1080)
                .image_id(42)
                .placement_id(1)
                .z_index(-1)
                .display_area(80, 24)
                .source_rect(0, 0, 800, 600)
                .quiet(2)
                .build()
        })
    });

    // Animation frame command
    group.bench_function("animation_frame", |b| {
        b.iter(|| {
            Command::builder()
                .action(Action::Frame)
                .image_id(1)
                .frame_number(2)
                .frame_gap(16)
                .build()
        })
    });

    // Delete command
    group.bench_function("delete_command", |b| {
        b.iter(|| {
            Command::builder()
                .action(Action::Delete)
                .delete_target(DeleteTarget::ById { free_data: true })
                .image_id(1)
                .build()
        })
    });

    group.finish();
}

// =============================================================================
// Serialization Benchmarks
// =============================================================================

fn bench_serialize_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_small");

    // Small PNG (1KB)
    let png_data = generate_png_like_data(1024);
    group.throughput(Throughput::Bytes(png_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();

    group.bench_function("png_1kb", |b| {
        b.iter(|| black_box(cmd.serialize(&png_data).unwrap()))
    });

    // Small RGBA (100x100 = 40KB)
    let rgba_data = generate_rgba_data(100, 100);
    group.throughput(Throughput::Bytes(rgba_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Rgba)
        .dimensions(100, 100)
        .quiet(2)
        .build();

    group.bench_function("rgba_100x100", |b| {
        b.iter(|| black_box(cmd.serialize(&rgba_data).unwrap()))
    });

    group.finish();
}

fn bench_serialize_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_medium");

    // Medium PNG (50KB)
    let png_data = generate_png_like_data(50 * 1024);
    group.throughput(Throughput::Bytes(png_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();

    group.bench_function("png_50kb", |b| {
        b.iter(|| black_box(cmd.serialize(&png_data).unwrap()))
    });

    // Medium RGBA (256x256 = 256KB)
    let rgba_data = generate_rgba_data(256, 256);
    group.throughput(Throughput::Bytes(rgba_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Rgba)
        .dimensions(256, 256)
        .quiet(2)
        .build();

    group.bench_function("rgba_256x256", |b| {
        b.iter(|| black_box(cmd.serialize(&rgba_data).unwrap()))
    });

    group.finish();
}

fn bench_serialize_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_large");

    // Large PNG (500KB)
    let png_data = generate_png_like_data(500 * 1024);
    group.throughput(Throughput::Bytes(png_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();

    group.bench_function("png_500kb", |b| {
        b.iter(|| black_box(cmd.serialize(&png_data).unwrap()))
    });

    // Large RGBA (512x512 = 1MB)
    let rgba_data = generate_rgba_data(512, 512);
    group.throughput(Throughput::Bytes(rgba_data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Rgba)
        .dimensions(512, 512)
        .quiet(2)
        .build();

    group.bench_function("rgba_512x512", |b| {
        b.iter(|| black_box(cmd.serialize(&rgba_data).unwrap()))
    });

    group.finish();
}

// =============================================================================
// Chunked Serialization Benchmarks
// =============================================================================

fn bench_chunked_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunked_serialize");

    // 4KB
    let data_4k = generate_png_like_data(4 * 1024);
    group.throughput(Throughput::Bytes(4 * 1024));
    let cmd_4k = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();
    group.bench_function("size_4kb", |b| {
        b.iter(|| {
            let serializer = cmd_4k.serialize_chunked(&data_4k).unwrap();
            let chunks: Vec<String> = serializer.collect();
            black_box(chunks)
        })
    });

    // 16KB
    let data_16k = generate_png_like_data(16 * 1024);
    group.throughput(Throughput::Bytes(16 * 1024));
    let cmd_16k = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();
    group.bench_function("size_16kb", |b| {
        b.iter(|| {
            let serializer = cmd_16k.serialize_chunked(&data_16k).unwrap();
            let chunks: Vec<String> = serializer.collect();
            black_box(chunks)
        })
    });

    // 64KB
    let data_64k = generate_png_like_data(64 * 1024);
    group.throughput(Throughput::Bytes(64 * 1024));
    let cmd_64k = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();
    group.bench_function("size_64kb", |b| {
        b.iter(|| {
            let serializer = cmd_64k.serialize_chunked(&data_64k).unwrap();
            let chunks: Vec<String> = serializer.collect();
            black_box(chunks)
        })
    });

    // 256KB
    let data_256k = generate_png_like_data(256 * 1024);
    group.throughput(Throughput::Bytes(256 * 1024));
    let cmd_256k = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();
    group.bench_function("size_256kb", |b| {
        b.iter(|| {
            let serializer = cmd_256k.serialize_chunked(&data_256k).unwrap();
            let chunks: Vec<String> = serializer.collect();
            black_box(chunks)
        })
    });

    // 1MB
    let data_1m = generate_png_like_data(1024 * 1024);
    group.throughput(Throughput::Bytes(1024 * 1024));
    let cmd_1m = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();
    group.bench_function("size_1mb", |b| {
        b.iter(|| {
            let serializer = cmd_1m.serialize_chunked(&data_1m).unwrap();
            let chunks: Vec<String> = serializer.collect();
            black_box(chunks)
        })
    });

    group.finish();
}

fn bench_chunked_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunked_iteration");

    // 100KB data - measure just the iteration overhead
    let data = generate_png_like_data(100 * 1024);
    group.throughput(Throughput::Bytes(data.len() as u64));

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .quiet(2)
        .build();

    group.bench_function("iterate_chunks_100kb", |b| {
        b.iter(|| {
            let serializer = cmd.serialize_chunked(&data).unwrap();
            let count = serializer.count();
            black_box(count)
        })
    });

    group.finish();
}

// =============================================================================
// Response Parsing Benchmarks
// =============================================================================

fn bench_response_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_parsing");

    // OK response
    let ok_response = b"\x1b_Gi=42;OK\x1b\\";
    group.bench_function("parse_ok", |b| {
        b.iter(|| black_box(Response::parse(ok_response).unwrap()))
    });

    // OK with placement
    let ok_placement = b"\x1b_Gi=42,p=7;OK\x1b\\";
    group.bench_function("parse_ok_placement", |b| {
        b.iter(|| black_box(Response::parse(ok_placement).unwrap()))
    });

    // OK with image number
    let ok_number = b"\x1b_Gi=99,I=13;OK\x1b\\";
    group.bench_function("parse_ok_number", |b| {
        b.iter(|| black_box(Response::parse(ok_number).unwrap()))
    });

    // Error response
    let error_response = b"\x1b_Gi=42;ENOENT:Image not found\x1b\\";
    group.bench_function("parse_error", |b| {
        b.iter(|| black_box(Response::parse(error_response).unwrap()))
    });

    group.finish();
}

// =============================================================================
// Convenience Function Benchmarks
// =============================================================================

fn bench_convenience_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("convenience_functions");

    // transmit_png
    let png_data = generate_png_like_data(50 * 1024);
    group.throughput(Throughput::Bytes(png_data.len() as u64));

    group.bench_function("transmit_png_50kb", |b| {
        b.iter(|| {
            let chunks = Command::transmit_png(&png_data).unwrap();
            black_box(chunks)
        })
    });

    // transmit_rgba
    let rgba_data = generate_rgba_data(256, 256);
    group.throughput(Throughput::Bytes(rgba_data.len() as u64));

    group.bench_function("transmit_rgba_256x256", |b| {
        b.iter(|| {
            let chunks = Command::transmit_rgba(&rgba_data, 256, 256).unwrap();
            black_box(chunks)
        })
    });

    // transmit_rgb
    let rgb_data = generate_rgb_data(256, 256);
    group.throughput(Throughput::Bytes(rgb_data.len() as u64));

    group.bench_function("transmit_rgb_256x256", |b| {
        b.iter(|| {
            let chunks = Command::transmit_rgb(&rgb_data, 256, 256).unwrap();
            black_box(chunks)
        })
    });

    group.finish();
}

// =============================================================================
// Control Data Building Benchmarks
// =============================================================================

fn bench_control_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_data");

    // Simple command
    let simple_cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Png)
        .build();

    group.bench_function("simple_control_data", |b| {
        b.iter(|| {
            let data = simple_cmd.clone().serialize(&[]).unwrap();
            black_box(data)
        })
    });

    // Complex command with many fields
    let complex_cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Rgba)
        .dimensions(1920, 1080)
        .image_id(42)
        .placement_id(7)
        .z_index(-100)
        .display_area(120, 40)
        .source_rect(100, 50, 800, 600)
        .cell_offset(2, 3)
        .quiet(2)
        .cursor_policy(kitty_graphics_protocol::CursorPolicy::NoMove)
        .build();

    group.bench_function("complex_control_data", |b| {
        b.iter(|| {
            let data = complex_cmd.clone().serialize(&[]).unwrap();
            black_box(data)
        })
    });

    group.finish();
}

// =============================================================================
// Base64 Encoding Benchmarks (for comparison)
// =============================================================================

fn bench_base64_encoding(c: &mut Criterion) {
    use base64::{engine::general_purpose::STANDARD, Engine};

    let mut group = c.benchmark_group("base64_encoding");

    // 1KB
    let data_1k = generate_rgba_data(1, 256);
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("encode_1kb", |b| {
        b.iter(|| black_box(STANDARD.encode(&data_1k)))
    });

    // 10KB
    let data_10k = generate_rgba_data(1, 2560);
    group.throughput(Throughput::Bytes(10 * 1024));
    group.bench_function("encode_10kb", |b| {
        b.iter(|| black_box(STANDARD.encode(&data_10k)))
    });

    // 100KB
    let data_100k = generate_rgba_data(1, 25600);
    group.throughput(Throughput::Bytes(100 * 1024));
    group.bench_function("encode_100kb", |b| {
        b.iter(|| black_box(STANDARD.encode(&data_100k)))
    });

    // 1MB
    let data_1m = generate_rgba_data(1, 262144);
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("encode_1mb", |b| {
        b.iter(|| black_box(STANDARD.encode(&data_1m)))
    });

    group.finish();
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(std::time::Duration::from_secs(5));
    targets =
        bench_command_builder,
        bench_serialize_small,
        bench_serialize_medium,
        bench_serialize_large,
        bench_chunked_serialize,
        bench_chunked_iteration,
        bench_response_parsing,
        bench_convenience_functions,
        bench_control_data,
        bench_base64_encoding
}

criterion_main!(benches);
