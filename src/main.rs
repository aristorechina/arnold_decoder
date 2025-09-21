use anyhow::{bail, Context, Result};
use image::RgbImage;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::mem;

fn apply_transform_to_buffer(src: &RgbImage, dst: &mut RgbImage, a: i64, b: i64) {
    let (width, height) = src.dimensions();
    let n = height as i64;

    // 并行遍历目标图像的每一行
    dst.par_chunks_mut(width as usize * 3).enumerate().for_each(|(row_idx, row_slice)| {
        let new_row_i64 = row_idx as i64;
        for col_idx in 0..width {
            let new_col_i64 = col_idx as i64;

            let old_row = (new_row_i64 + b * new_col_i64).rem_euclid(n) as u32;
            let old_col = (a * new_row_i64 + (a * b + 1) * new_col_i64).rem_euclid(n) as u32;

            let pixel = *src.get_pixel(old_col, old_row);

            let pixel_slice = &mut row_slice[(col_idx * 3) as usize..(col_idx * 3 + 3) as usize];
            pixel_slice[0] = pixel[0];
            pixel_slice[1] = pixel[1];
            pixel_slice[2] = pixel[2];
        }
    });
}

// 通过交换缓冲区避免在循环中重复分配内存
fn arnold_decode(image: &RgbImage, shuffle_times: u32, a: i64, b: i64) -> RgbImage {
    if shuffle_times == 0 {
        return image.clone();
    }
    
    let (width, height) = image.dimensions();

    let mut buffer1 = image.clone();
    let mut buffer2 = RgbImage::new(width, height);

    let mut src = &mut buffer1;
    let mut dst = &mut buffer2;

    for _ in 0..shuffle_times {
        apply_transform_to_buffer(src, dst, a, b);
        mem::swap(&mut src, &mut dst);
    }

    src.clone()
}


fn parse_path_input(input: &str) -> PathBuf {
    let trimmed = input.trim();
    let dequoted = trimmed.trim_matches(|c| c == '\"' || c == '\'');
    let normalized = dequoted.replace('\\', "/");
    PathBuf::from(normalized)
}

fn read_line_from_stdin() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn get_user_range(prompt: &str) -> Result<std::ops::Range<i64>> {
    loop {
        print!("{}", prompt);
        io::stdout().flush()?;
        let input = read_line_from_stdin()?;
        
        if let Ok(single_val) = input.parse::<i64>() {
             return Ok(single_val..(single_val + 1));
        }

        let parts: Vec<&str> = input.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].trim().parse(), parts[1].trim().parse()) {
                if start <= end {
                    return Ok(start..(end + 1));
                }
            }
        }
        println!("🤔 格式错误，请输入单个数字 (如 '8') 或范围 (如 '0-10')");
    }
}

fn main() -> Result<()> {
    println!(r"");
    println!(r"================================================================================================================");
    println!(r" █████╗ ██████╗ ███╗   ██╗ ██████╗ ██╗     ██████╗     ██████╗ ███████╗ ██████╗ ██████╗ ██████╗ ███████╗██████╗ ");
    println!(r"██╔══██╗██╔══██╗████╗  ██║██╔═══██╗██║     ██╔══██╗    ██╔══██╗██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝██╔══██╗");
    println!(r"███████║██████╔╝██╔██╗ ██║██║   ██║██║     ██║  ██║    ██║  ██║█████╗  ██║     ██║   ██║██║  ██║█████╗  ██████╔╝");
    println!(r"██╔══██║██╔══██╗██║╚██╗██║██║   ██║██║     ██║  ██║    ██║  ██║██╔══╝  ██║     ██║   ██║██║  ██║██╔══╝  ██╔══██╗");
    println!(r"██╔══██║██╔══██╗██║╚██╗██║██║   ██║██║     ██║  ██║    ██║  ██║██╔══╝  ██║     ██║   ██║██║  ██║██╔══╝  ██╔══██╗");
    println!(r"██║  ██║██║  ██║██║ ╚████║╚██████╔╝███████╗██████╔╝    ██████╔╝███████╗╚██████╗╚██████╔╝██████╔╝███████╗██║  ██║");
    println!(r"╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝╚═════╝     ╚═════╝ ╚══════╝ ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝");
    println!();
    println!(r"                                                Author: Aristore                                                ");
    println!(r"================================================================================================================");
    println!();

    fn pause_before_exit() {
        print!("\nPress Enter to exit...");
        io::stdout().flush().unwrap_or_default();
        let _ = read_line_from_stdin();
    }

    let image_path = loop {
        print!("📂 请输入图片路径: ");
        io::stdout().flush()?;
        let input = read_line_from_stdin()?;
        let path = parse_path_input(&input);
        if path.exists() {
            break path;
        } else {
            println!("❌ 文件不存在: {:?}", path);
        }
    };
    
    let encoded_image = image::open(&image_path)
        .with_context(|| format!("❌ 无法读取图像文件: {:?}", image_path))?
        .to_rgb8();

    if encoded_image.width() != encoded_image.height() {
        bail!("❌ Arnold变换要求图像为正方形，但当前图像尺寸为 {}x{}", encoded_image.width(), encoded_image.height());
    }

    println!("✅ 图片加载成功: {}x{}", encoded_image.width(), encoded_image.height());
    println!("--------------------------------------");

    println!("🔢 请输入要爆破的参数范围");
    let shuffle_times_range = get_user_range("   - 变换次数 (例如 '8' 或 '0-10'): ")?;
    let a_values_range = get_user_range("   - 参数 a   (例如 '8' 或 '0-10'): ")?;
    let b_values_range = get_user_range("   - 参数 b   (例如 '8' 或 '0-10'): ")?;
    println!("--------------------------------------");

    let parent_dir = image_path.parent().unwrap_or_else(|| Path::new("."));
    let output_dir = parent_dir.join("Arnold_Output");
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("❌ 无法创建输出目录: {:?}", output_dir))?;
    
    println!("🚀 输出结果将保存在: {:?}", output_dir);
    println!();

    let mut params = Vec::new();
    for st in shuffle_times_range {
        for a in a_values_range.clone() {
            for b in b_values_range.clone() {
                params.push((st as u32, a, b));
            }
        }
    }
    
    if params.is_empty() {
        println!("🤷‍♀️ 没有有效的参数组合");
        return Ok(());
    }

    let bar_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)  ETA: {eta}")
        .unwrap()
        .progress_chars("#>-");
    let bar = ProgressBar::new(params.len() as u64).with_style(bar_style);

    let start_time = std::time::Instant::now();

    params
        .into_par_iter()
        .progress_with(bar)
        .for_each(|(shuffle_times, a, b)| {
            let decoded_image = arnold_decode(&encoded_image, shuffle_times, a, b);
            let output_filename = format!("{}_{}_{}.png", shuffle_times, a, b);
            let output_path = output_dir.join(output_filename);
            decoded_image.save(output_path).ok();
        });

    let duration = start_time.elapsed();
    println!("\n⏱️ 用时: {:.2} 秒", duration.as_secs_f64());

    println!("🎉 处理完成");
    pause_before_exit();
    Ok(())
}