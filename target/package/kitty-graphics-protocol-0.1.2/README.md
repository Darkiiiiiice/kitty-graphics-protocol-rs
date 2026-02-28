# Kitty Graphics Protocol

[![Crates.io](https://img.shields.io/crates/v/kitty-graphics-protocol.svg)](https://crates.io/crates/kitty-graphics-protocol)
[![Documentation](https://docs.rs/kitty-graphics-protocol/badge.svg)](https://docs.rs/kitty-graphics-protocol)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个完整的 Rust 实现的 [Kitty 终端图形协议](https://sw.kovidgoyal.net/kitty/graphics-protocol/) 库，允许你在支持该协议的终端中显示图像和动画。

## 功能特性

- ✅ 完整支持所有图形协议命令
- ✅ 支持 RGB、RGBA 和 PNG 图像格式
- ✅ 大图像分块传输
- ✅ 动画支持
- ✅ Unicode 占位符支持
- ✅ 终端窗口大小检测
- ✅ 协议支持检测
- ✅ 零依赖图像显示（PNG 格式）
- ✅ 类型安全的 Builder 模式 API

## 兼容终端

- [Kitty](https://sw.kovidgoyal.net/kitty/) - 原生支持
- [Konsole](https://konsole.kde.org/) - 23.04+ 版本支持
- [WezTerm](https://wezfurlong.org/wezterm/) - 支持
- [foot](https://codeberg.org/dnkl/foot) - 支持
- [Alacritty](https://github.com/alacritty/alacritty) - 通过补丁支持

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
kitty-graphics-protocol = "0.1"
```

## 快速开始

### 显示 PNG 图片

```rust
use kitty_graphics_protocol::display_png;

// 最简单的方式 - 直接显示 PNG 文件
display_png("image.png").unwrap();
```

### 显示内存中的 PNG 数据

```rust
use kitty_graphics_protocol::display_png_data;

let png_data = std::fs::read("image.png").unwrap();
display_png_data(&png_data).unwrap();
```

### 显示原始 RGB/RGBA 数据

```rust
use kitty_graphics_protocol::ImageDisplay;

let display = ImageDisplay::new();

// 显示 RGB 数据 (每像素 3 字节)
let rgb_data: Vec<u8> = vec![/* R, G, B, R, G, B, ... */];
display.display_rgb(&rgb_data, width, height).unwrap();

// 显示 RGBA 数据 (每像素 4 字节)
let rgba_data: Vec<u8> = vec![/* R, G, B, A, R, G, B, A, ... */];
display.display_rgba(&rgba_data, width, height).unwrap();
```

## 详细用法

### 1. 高级 ImageDisplay 接口

`ImageDisplay` 提供了一个高级接口来管理图像显示：

```rust
use kitty_graphics_protocol::ImageDisplay;

let display = ImageDisplay::new()
    .quiet(2);  // 抑制所有响应消息

// 显示 PNG 文件
display.display_png_file("photo.png").unwrap();

// 显示 PNG 数据
let png_data = std::fs::read("image.png").unwrap();
display.display_png(&png_data).unwrap();

// 显示原始像素数据
display.display_rgb(&rgb_data, width, height).unwrap();
display.display_rgba(&rgba_data, width, height).unwrap();

// 清除所有可见图像
display.clear_all().unwrap();
```

### 2. 图片 ID 管理（传输一次，多次显示）

```rust
use kitty_graphics_protocol::ImageDisplay;

let display = ImageDisplay::new();
let png_data = std::fs::read("image.png").unwrap();

// 传输图片但不显示，分配 ID 为 123
display.transmit_png(&png_data, 123).unwrap();

// 在不同位置多次显示同一图片
display.place_image(123, 10, 5).unwrap();  // 10 列, 5 行
display.place_image(123, 20, 5).unwrap();  // 20 列, 5 行

// 清除图片
display.clear_all().unwrap();
```

### 3. 使用 Command Builder（低级 API）

对于完全控制，使用 `Command` builder：

```rust
use kitty_graphics_protocol::{Command, Action, ImageFormat};
use std::io::Write;

// 构建命令
let cmd = Command::builder()
    .action(Action::TransmitAndDisplay)
    .format(ImageFormat::Png)
    .quiet(2)                    // 抑制响应
    .z_index(0)                  // 设置 z-index
    .display_area(20, 10)        // 显示区域: 20 列 x 10 行
    .build();

// 序列化为转义序列块
let png_data = std::fs::read("image.png").unwrap();
let chunks: Vec<String> = cmd.serialize_chunked(&png_data).unwrap().collect();

// 输出到终端
let mut stdout = std::io::stdout().lock();
for chunk in chunks {
    stdout.write_all(chunk.as_bytes()).unwrap();
}
stdout.flush().unwrap();
```

### 4. 显示原始像素数据

```rust
use kitty_graphics_protocol::{Command, Action, ImageFormat};

// RGB 格式 (每像素 3 字节: R, G, B)
let width = 100;
let height = 50;
let rgb_data: Vec<u8> = (0..width * height * 3)
    .map(|i| (i % 256) as u8)
    .collect();

let cmd = Command::builder()
    .action(Action::TransmitAndDisplay)
    .format(ImageFormat::Rgb)
    .dimensions(width, height)
    .build();

// 输出
let chunks: Vec<String> = cmd.serialize_chunked(&rgb_data).unwrap().collect();
for chunk in chunks {
    print!("{}", chunk);
}
```

### 5. 删除图像

```rust
use kitty_graphics_protocol::{Command, clear_all_images, DeleteTarget};

// 方式 1: 清除所有图像
clear_all_images().unwrap();

// 方式 2: 使用 Command
let cmd = Command::delete_all();
let seq = cmd.serialize(&[]).unwrap();
print!("{}", seq);

// 方式 3: 按 ID 删除
let cmd = Command::delete_by_id(123);
let seq = cmd.serialize(&[]).unwrap();
print!("{}", seq);

// 方式 4: 使用 DeleteTarget
let cmd = Command::builder()
    .action(Action::Delete)
    .delete_target(DeleteTarget::ByColumn { free_data: true })
    .build();
```

### 6. 终端信息查询

```rust
use kitty_graphics_protocol::{check_protocol_support, get_window_size};

// 检查协议支持
match check_protocol_support() {
    Ok(true) => println!("支持 Kitty 图形协议"),
    Ok(false) => println!("不支持 Kitty 图形协议"),
    Err(e) => println!("检测失败: {}", e),
}

// 获取窗口大小
match get_window_size() {
    Ok(size) => {
        println!("窗口: {}x{} 像素", size.width, size.height);
        println!("单元格: {}x{} 像素", size.cell_width(), size.cell_height());
        println!("列数: {}, 行数: {}", size.cols, size.rows);

        // 计算显示图片需要的单元格数
        let (cols, rows) = size.cells_for_image(800, 600);
        println!("800x600 图片需要 {} 列 x {} 行", cols, rows);
    }
    Err(e) => println!("获取失败: {}", e),
}
```

### 7. 动画支持

```rust
use kitty_graphics_protocol::{Command, Action, AnimationControl};

// 停止动画
let cmd = Command::builder()
    .action(Action::AnimationControl)
    .animation_control(AnimationControl::Stop)
    .image_id(1)
    .build();

// 运行动画
let cmd = Command::builder()
    .action(Action::AnimationControl)
    .animation_control(AnimationControl::Run)
    .image_id(1)
    .loop_count(1)  // 无限循环
    .frame_gap(100) // 帧间隔 100ms
    .build();
```

## API 参考

### 主要类型

| 类型 | 描述 |
|------|------|
| `ImageDisplay` | 高级图像显示接口 |
| `Command` | 图形协议命令 |
| `CommandBuilder` | 命令构建器 |
| `WindowSize` | 终端窗口大小信息 |

### 快捷函数

| 函数 | 描述 |
|------|------|
| `display_png(path)` | 显示 PNG 文件 |
| `display_png_data(data)` | 显示 PNG 数据 |
| `clear_all_images()` | 清除所有图像 |
| `check_protocol_support()` | 检查协议支持 |
| `get_window_size()` | 获取窗口大小 |

### 枚举类型

| 枚举 | 描述 |
|------|------|
| `Action` | 命令动作类型 |
| `ImageFormat` | 图像格式 (RGB=24, RGBA=32, PNG=100) |
| `TransmissionMedium` | 传输方式 |
| `DeleteTarget` | 删除目标 |
| `AnimationControl` | 动画控制 |
| `Compression` | 压缩算法 |
| `CursorPolicy` | 光标移动策略 |

## 示例

项目包含两个示例程序：

### 基础示例 - 显示图片文件

```bash
cargo run --example display_image <图片路径>
```

### 高级示例 - 程序化生成图像

```bash
cargo run --example advanced_display
```

## 运行示例

```bash
# 克隆仓库
git clone https://github.com/user/kitty-graphics-protocol.git
cd kitty-graphics-protocol

# 运行基础示例
cargo run --example display_image test.png

# 运行高级示例
cargo run --example advanced_display
```

## 注意事项

1. **终端兼容性**: 确保在支持 Kitty 图形协议的终端中运行
2. **图像格式**: 原生支持 PNG 格式；其他格式需转换为 RGB/RGBA 原始数据
3. **大图像**: 大图像会自动分块传输，无需手动处理
4. **图像清理**: 程序退出前建议调用 `clear_all_images()` 清理显示的图像

## 错误处理

```rust
use kitty_graphics_protocol::{display_png, Error};

match display_png("image.png") {
    Ok(_) => println!("显示成功"),
    Err(Error::Io(e)) => eprintln!("IO 错误: {}", e),
    Err(Error::InvalidDimensions { width, height }) => {
        eprintln!("无效尺寸: {}x{}", width, height);
    }
    Err(e) => eprintln!("其他错误: {}", e),
}
```

## 性能

- 使用 Base64 编码传输数据
- 支持分块传输大图像（默认 4KB 块）
- 零拷贝设计，最小化内存分配

## 许可证

MIT License

## 参考资料

- [Kitty Graphics Protocol 规范](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- [Kitty Terminal](https://sw.kovidgoyal.net/kitty/)

## 贡献

欢迎提交 Issue 和 Pull Request！
