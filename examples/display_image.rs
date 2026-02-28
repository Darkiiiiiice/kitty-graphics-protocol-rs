//! Kitty Graphics Protocol 示例程序
//!
//! 在 Kitty 终端中显示图片的示例
//!
//! 用法:
//!   cargo run --example display_image <图片路径>
//!   cargo run --example display_image -- --help

use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use kitty_graphics_protocol::{check_protocol_support, clear_all_images, display_png_data};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let path = &args[1];

    // 检查是否支持 Kitty 图形协议
    println!("检查 Kitty 图形协议支持...");
    match check_protocol_support() {
        Ok(supported) => {
            if !supported {
                eprintln!("错误: 当前终端不支持 Kitty 图形协议");
                eprintln!("请在 Kitty 终端或兼容的终端中运行此程序");
                process::exit(1);
            }
            println!("✓ 终端支持 Kitty 图形协议");
        }
        Err(e) => {
            eprintln!("警告: 无法检测协议支持: {}", e);
            println!("继续尝试显示图片...");
        }
    }

    // 显示图片
    match display_image(path) {
        Ok(_) => {
            println!("\n图片显示成功！");
            println!("等待 5 秒后清除图片...");
            thread::sleep(Duration::from_secs(5));

            // 清除显示的图片
            if let Err(e) = clear_all_images() {
                eprintln!("清除图片失败: {}", e);
            }
        }
        Err(e) => {
            eprintln!("显示图片失败: {}", e);
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    println!("Kitty 终端图片显示示例");
    println!();
    println!("用法:");
    println!("  {} <图片路径>", program);
    println!();
    println!("支持的格式:");
    println!("  PNG (推荐)");
    println!();
    println!("示例:");
    println!("  {} photo.png", program);
    println!("  {} /path/to/image.png", program);
}

/// 图像格式类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    WebP,
    Unknown,
}

fn detect_format(data: &[u8]) -> ImageFormat {
    if data.len() < 8 {
        return ImageFormat::Unknown;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return ImageFormat::Png;
    }

    // JPEG: FF D8 FF
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return ImageFormat::Jpeg;
    }

    // GIF: 47 49 46 38
    if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x38 {
        return ImageFormat::Gif;
    }

    // BMP: 42 4D
    if data[0] == 0x42 && data[1] == 0x4D {
        return ImageFormat::Bmp;
    }

    // WebP: 52 49 46 46 ... 57 45 42 50
    if data[0] == 0x52
        && data[1] == 0x49
        && data[2] == 0x46
        && data[3] == 0x46
        && data.len() >= 12
        && data[8] == 0x57
        && data[9] == 0x45
        && data[10] == 0x42
        && data[11] == 0x50
    {
        return ImageFormat::WebP;
    }

    ImageFormat::Unknown
}

fn display_image(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 读取图片文件
    let data = std::fs::read(path)?;

    // 检测图像格式
    let format = detect_format(&data);

    match format {
        ImageFormat::Png => {
            println!("✓ 检测到 PNG 格式图片");
            display_png_data(&data)?;
        }
        ImageFormat::Jpeg => {
            eprintln!("✗ 错误: JPEG 格式不被原生支持");
            eprintln!();
            eprintln!("Kitty 图形协议原生只支持 PNG 格式。");
            eprintln!("请将图片转换为 PNG 格式后重试：");
            eprintln!();
            eprintln!("  # 使用 ImageMagick 转换");
            eprintln!("  convert \"{}\" output.png", path);
            eprintln!();
            eprintln!("  # 或使用 ffmpeg");
            eprintln!("  ffmpeg -i \"{}\" output.png", path);
            return Err("不支持的图像格式: JPEG".into());
        }
        ImageFormat::Gif => {
            eprintln!("✗ 错误: GIF 格式不被原生支持");
            eprintln!("请将图片转换为 PNG 格式后重试");
            return Err("不支持的图像格式: GIF".into());
        }
        ImageFormat::Bmp => {
            eprintln!("✗ 错误: BMP 格式不被原生支持");
            eprintln!("请将图片转换为 PNG 格式后重试");
            return Err("不支持的图像格式: BMP".into());
        }
        ImageFormat::WebP => {
            eprintln!("✗ 错误: WebP 格式不被原生支持");
            eprintln!("请将图片转换为 PNG 格式后重试");
            return Err("不支持的图像格式: WebP".into());
        }
        ImageFormat::Unknown => {
            eprintln!("✗ 错误: 无法识别的图像格式");
            eprintln!("请确保文件是有效的 PNG 图像");
            return Err("无法识别的图像格式".into());
        }
    }

    Ok(())
}
