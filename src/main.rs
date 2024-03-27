use std::{array, collections::BTreeMap, env::{self, args}, fs::File, io::{BufRead, BufReader}, path::{Path, PathBuf}};

use image::{imageops::{resize, FilterType}, GenericImageView, Pixel, Rgb, Rgba, RgbaImage};
use lab::Lab;

fn main() {
    let mut args = args().skip(1);
    let img_path: PathBuf = args.next().expect("image").into();
    let pixels_pr_bead = args.next().map(|s| s.parse().expect("number")).unwrap_or(1);
    let output_scale = args.next().map(|s| s.parse().expect("number"));

    let palette = read_palette();

    let (frequency, beads) = create_beads(&img_path, pixels_pr_bead, &palette);

    let mut total_pearls = 0;
    for (name, pearls) in frequency {
        total_pearls += pearls;
        println!("{name}: {pearls}");
    }
    println!(" Total: {total_pearls}");

    output(beads, &img_path, output_scale);
}

fn read_palette() -> Vec<(Box<str>, Rgb<u8>)> {
    let mut palette = Vec::new();
    for line in BufReader::new(File::open("palette.txt").unwrap()).lines() {
        let line = line.unwrap();
        let line = line.trim();
        let (name, hex) = line.split_once(' ').unwrap();
        let hex = u32::from_str_radix(hex, 16).unwrap();
        palette.push((name.into(), make_rgb(hex)));
    }
    palette
}

fn make_rgb(rgb: u32) -> Rgb<u8> {
    let r = (rgb >> 16) as u8;
    let g = (rgb >> 8) as u8;
    let b = rgb as u8;
    Rgb([r, g, b])
}

fn create_beads<'a, 'b>(img_path: &'b Path, pixels_pr_bead: u32, palette: &'a [(Box<str>, Rgb<u8>)]) -> (BTreeMap<&'a str, u32>, RgbaImage) {
    let img = image::open(img_path).unwrap();
    let width = img.width() / pixels_pr_bead;
    let height = img.height() / pixels_pr_bead;

    let mut img = img.resize_exact(width, height, FilterType::CatmullRom).into_rgba8();
    let mut frequency = BTreeMap::new();

    let distance = env::var("DIST").map(|s| match &*s {
        "rgb" => distance_rgb,
        "lab" => distance_lab,
        _ => unimplemented!(),
    }).unwrap_or(distance_rgb);

    img.pixels_mut().for_each(|p| { 
        let Rgba([r, g, b, a]) = *p;
        if a < 128 {
            *p = Rgba([255, 255, 255, 0]);
        } else {
            let target = Rgb([r, g, b]);

            let mut colour = target;
            let mut best_dist = f32::INFINITY;
            let mut chosen_name = "";
            for &(ref name, candidate_colour) in palette.iter() {
                let dist = distance(candidate_colour, target);
                if dist < best_dist {
                    best_dist = dist;
                    colour = candidate_colour;
                    chosen_name = &**name;
                }
            }

            *frequency.entry(chosen_name).or_insert(0u32) += 1;
            *p = colour.to_rgba();
        }
    });

    (frequency, img)
}

fn distance_rgb(a: Rgb<u8>, b: Rgb<u8>) -> f32 {
    a.0.into_iter().zip(b.0.into_iter())
        .map(|(a, b)| {
            let d = a as f32 - b as f32;
            d * d
        })
        .sum()
}
fn distance_lab(a: Rgb<u8>, b: Rgb<u8>) -> f32 {
    let a = Lab::from_rgb(&a.0);
    let b = Lab::from_rgb(&b.0);

    a.squared_distance(&b)
}

fn output(beads: RgbaImage, img_path: &Path, output_scale: Option<u16>) {
    let out_path = img_path.with_extension("perlur.png");

    let Some(output_scale) = output_scale else {
        return show_pearls(beads, out_path);
    };

    let img = resize(&beads, beads.width()*output_scale as u32, beads.height() * output_scale as u32, FilterType::Nearest);
    img.save(out_path).unwrap();
}

fn show_pearls(beads: RgbaImage, out_path: PathBuf) {
    let perla = image::open("perla.png").unwrap();
    let (pw, ph) = perla.dimensions();

    RgbaImage::from_par_fn(beads.width() * pw, beads.height() * ph, |x, y| {
        let (ox, px) = (x / pw, x % pw);
        let (oy, py) = (y / pw, y % pw);

        let pc = perla.get_pixel(px, py);
        let c = *beads.get_pixel(ox, oy);

        mul_rgba(c, pc)
    }).save(out_path).unwrap();
}

fn mul_rgba(a: Rgba<u8>, b: Rgba<u8>) -> Rgba<u8> {
    Rgba(array::from_fn(|i| {
        ((a.0[i] as u16 * b.0[i] as u16) / 255) as u8
    }))
}
