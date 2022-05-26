use std::{env, fs::File, io::Read, process::exit};

use crate::color::Color;

const HEADER: [u8; 8] = [102, 97, 114, 98, 102, 101, 108, 100];

pub fn dither<R: std::io::Read, W: std::io::Write>(stdin: &mut R, stdout: &mut W) {
    let mut buffer = [0; 8];

    stdin.read_exact(&mut buffer).unwrap();

    if buffer != HEADER {
        eprintln!("Wrong header!");
        exit(1);
    }

    let mut bwidth = [0; 4];
    let mut bheight = [0; 4];
    stdin.read_exact(&mut bwidth).unwrap();
    stdin.read_exact(&mut bheight).unwrap();
    let width = u32::from_be_bytes(bwidth) as usize;
    let height = u32::from_be_bytes(bheight) as usize;

    let args: Vec<String> = env::args().collect();
    let pal: Vec<Color>;
    match args.get(1) {
        Some(filename) => {
            pal = read_pal(filename);
        }
        None => {
            let max = u16::MAX as i32;
            pal = vec![
                Color { r: 0, g: 0, b: 0 },
                Color {
                    r: max,
                    g: max,
                    b: max,
                },
                Color { r: max, g: 0, b: 0 },
                Color { r: 0, g: max, b: 0 },
                Color { r: 0, g: 0, b: max },
            ];
        }
    }

    let mut data = vec![];
    let mut out = vec![];
    let mut raw = Vec::new();
    stdin.read_to_end(&mut raw).unwrap();
    for y in 0..height {
        for x in 0..width {
            let start = ((y * width + x) * 8) as usize;
            let r = u16::from_be_bytes(raw[start..start + 2].try_into().unwrap()) as i32;
            let g = u16::from_be_bytes(raw[start + 2..start + 4].try_into().unwrap()) as i32;
            let b = u16::from_be_bytes(raw[start + 4..start + 6].try_into().unwrap()) as i32;
            let c = Color { r, g, b };
            data.push(c);
        }
    }

    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            let color = &data[i];
            let closest = closest_color(&pal, &color);
            let diff = *color - *closest;

            if x < width - 1 {
                data[i + 1] += (diff * 7) / 16;
            }

            if y < height - 1 {
                if x < width - 1 {
                    data[i + width + 1] += diff / 16;
                }
                if x > 0 {
                    data[i + width - 1] += (diff * 3) / 16;
                }
                data[i + width] += (diff * 5) / 16;
            }

            out.push(closest);
        }
    }

    stdout.write(&HEADER).unwrap();
    let ne_width = (width as u32).to_be_bytes();
    let ne_height = (height as u32).to_be_bytes();
    stdout.write(&ne_width).unwrap();
    stdout.write(&ne_height).unwrap();
    for v in out {
        let r = (v.r as u16).to_be_bytes();
        let g = (v.g as u16).to_be_bytes();
        let b = (v.b as u16).to_be_bytes();
        let a = u16::MAX.to_be_bytes();
        stdout.write(&r).unwrap();
        stdout.write(&g).unwrap();
        stdout.write(&b).unwrap();
        stdout.write(&a).unwrap();
    }
}

fn closest_color<'a>(pal: &'a Vec<Color>, src: &Color) -> &'a Color {
    let mut best_diff = f64::MAX;
    let mut best_col = &pal[0];
    for col in pal {
        let rmean = (src.r + col.r) as f64;
        let diff = *src - *col;
        let dr = diff.r.abs() as f64;
        let dg = diff.g.abs() as f64;
        let db = diff.b.abs() as f64;

        let max16 = u16::MAX as f64;
        let diff = ((512 as f64 + rmean / max16) * dr * dr
            + (1024 as f64) * dg * dg
            + (512 as f64 + (max16 - (1 as f64) - rmean) / max16) * db * db)
            .sqrt();
        if diff < best_diff {
            best_diff = diff;
            best_col = col;
        }
    }

    best_col
}

fn read_pal(filename: &String) -> Vec<Color> {
    let mut buf = String::new();
    let mut f = File::open(filename).unwrap();

    f.read_to_string(&mut buf).unwrap();

    let mul = (u8::MAX as i32) + 1;

    let mut pal = vec![];
    for line in buf.lines() {
        let rcol = hex::decode(line).unwrap();
        let col = Color {
            r: rcol[0] as i32,
            g: rcol[1] as i32,
            b: rcol[2] as i32,
        } * mul;
        pal.push(col);
    }

    pal
}
