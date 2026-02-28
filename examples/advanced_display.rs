//! Kitty Graphics Protocol 高级示例
//!
//! 展示更多 API 用法，包括:
//! - 使用 ImageDisplay 高级接口
//! - 显示 RGB/RGBA 原始数据
//! - 生成程序化图像
//!
//! 用法:
//!   cargo run --example advanced_display

use std::io::Write;
use std::thread;
use std::time::Duration;

use kitty_graphics_protocol::{
    Action, Command, ImageDisplay, ImageFormat,
    check_protocol_support, clear_all_images, get_window_size,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Kitty Graphics Protocol 高级示例 ===\n");

    // 1. 检查协议支持
    println!("1. 检查协议支持...");
    match check_protocol_support() {
        Ok(true) => println!("   ✓ 终端支持 Kitty 图形协议\n"),
        Ok(false) => {
            eprintln!("   ✗ 终端不支持 Kitty 图形协议");
            println!("   请在 Kitty 终端或兼容终端中运行\n");
        }
        Err(e) => {
            println!("   ? 无法检测: {}，继续尝试...\n", e);
        }
    }

    // 2. 获取终端窗口大小
    println!("2. 获取终端窗口大小...");
    match get_window_size() {
        Ok(size) => {
            println!("   窗口大小: {}x{} 像素", size.width, size.height);
            println!("   单元格大小: {}x{} 像素", size.cell_width(), size.cell_height());
            println!("   列数: {}, 行数: {}\n", size.cols, size.rows);
        }
        Err(e) => {
            println!("   无法获取窗口大小: {}\n", e);
        }
    }

    // 3. 生成并显示渐变图像
    println!("3. 显示程序化生成的渐变图像 (3秒)...");
    let display = ImageDisplay::new();
    let (width, height) = (100, 50);
    let gradient = generate_gradient(width, height);
    display.display_rgb(&gradient, width, height)?;
    thread::sleep(Duration::from_secs(3));
    clear_all_images()?;
    println!("   ✓ 渐变图像显示完成\n");

    // 4. 显示 RGBA 图像（带透明度）
    println!("4. 显示 RGBA 图像（带棋盘格透明度）(3秒)...");
    let (width, height) = (80, 40);
    let checkerboard = generate_checkerboard(width, height);
    display.display_rgba(&checkerboard, width, height)?;
    thread::sleep(Duration::from_secs(3));
    clear_all_images()?;
    println!("   ✓ RGBA 图像显示完成\n");

    // 5. 使用低级 API 直接构建命令
    println!("5. 使用低级 API 显示图像 (3秒)...");
    let (width, height) = (60, 30);
    let pattern = generate_diagonal_pattern(width, height);

    let cmd = Command::builder()
        .action(Action::TransmitAndDisplay)
        .format(ImageFormat::Rgb)
        .dimensions(width, height)
        .quiet(2)
        .build();

    let chunks: Vec<String> = cmd.serialize_chunked(&pattern)?.collect();
    let mut stdout = std::io::stdout().lock();
    for chunk in chunks {
        stdout.write_all(chunk.as_bytes())?;
    }
    stdout.flush()?;
    thread::sleep(Duration::from_secs(3));
    clear_all_images()?;
    println!("   ✓ 低级 API 显示完成\n");

    // 6. 显示彩色条纹
    println!("6. 显示彩色条纹 (3秒)...");
    let (width, height) = (120, 30);
    let stripes = generate_rainbow_stripes(width, height);
    display.display_rgb(&stripes, width, height)?;
    thread::sleep(Duration::from_secs(3));
    clear_all_images()?;
    println!("   ✓ 彩色条纹显示完成\n");

    println!("=== 示例完成 ===");
    Ok(())
}

/// 生成 RGB 渐变图像
fn generate_gradient(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128;
            data.push(r);
            data.push(g);
            data.push(b);
        }
    }
    data
}

/// 生成 RGBA 棋盘格图像
fn generate_checkerboard(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let cell_size = 10;

    for y in 0..height {
        for x in 0..width {
            let cell_x = x / cell_size;
            let cell_y = y / cell_size;
            let is_white = (cell_x + cell_y) % 2 == 0;

            data.push(if is_white { 255 } else { 100 }); // R
            data.push(if is_white { 255 } else { 100 }); // G
            data.push(if is_white { 255 } else { 100 }); // B
            data.push(if is_white { 255 } else { 180 }); // A
        }
    }
    data
}

/// 生成对角线图案
fn generate_diagonal_pattern(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let d = (x + y) % 20;
            let intensity = if d < 10 { d * 25 } else { (20 - d) * 25 };
            data.push(intensity as u8);
            data.push((255 - intensity) as u8);
            data.push(128);
        }
    }
    data
}

/// 生成彩虹条纹
fn generate_rainbow_stripes(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    // 彩虹颜色
    let colors: [(u8, u8, u8); 7] = [
        (255, 0, 0),    // 红
        (255, 127, 0),  // 橙
        (255, 255, 0),  // 黄
        (0, 255, 0),    // 绿
        (0, 0, 255),    // 蓝
        (75, 0, 130),   // 靛
        (148, 0, 211),  // 紫
    ];

    let stripe_width = width / colors.len() as u32;

    for _y in 0..height {
        for x in 0..width {
            let color_idx = (x / stripe_width).min((colors.len() - 1) as u32) as usize;
            let (r, g, b) = colors[color_idx];
            data.push(r);
            data.push(g);
            data.push(b);
        }
    }
    data
}
