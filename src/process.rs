use std::{array, collections::BTreeMap, path::Path};

use clap::ValueEnum;
use image::{
    imageops::{resize, FilterType},
    GenericImageView, Pixel, Rgb, Rgba, RgbaImage,
};
use lab::Lab;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DistanceMeasure {
    Rgb,
    Lab,
}
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DownscaleFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl From<DownscaleFilter> for FilterType {
    fn from(value: DownscaleFilter) -> Self {
        match value {
            DownscaleFilter::Nearest => FilterType::Nearest,
            DownscaleFilter::Triangle => FilterType::Triangle,
            DownscaleFilter::CatmullRom => FilterType::CatmullRom,
            DownscaleFilter::Gaussian => FilterType::Gaussian,
            DownscaleFilter::Lanczos3 => FilterType::Lanczos3,
        }
    }
}

pub fn create_beads<'a, 'b>(
    img_path: &'b Path,
    pixels_pr_bead: u32,
    palette: &'a [(Box<str>, Rgb<u8>)],
    distance: DistanceMeasure,
    filter: DownscaleFilter,
) -> (BTreeMap<&'a str, u32>, RgbaImage) {
    let img = image::open(img_path).unwrap();
    let width = img.width() / pixels_pr_bead;
    let height = img.height() / pixels_pr_bead;

    let mut img = img.resize_exact(width, height, filter.into()).into_rgba8();
    let mut frequency = BTreeMap::new();

    let distance = match distance {
        DistanceMeasure::Lab => distance_lab,
        DistanceMeasure::Rgb => distance_rgb,
    };

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
    a.0.into_iter()
        .zip(b.0.into_iter())
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

pub fn output(beads: RgbaImage, out_path: &Path, output_scale: Option<u32>, perla: &Path) {
    let Some(output_scale) = output_scale else {
        return show_pearls(beads, out_path, perla);
    };

    let img = resize(
        &beads,
        beads.width() * output_scale,
        beads.height() * output_scale,
        FilterType::Nearest,
    );
    img.save(out_path).unwrap();
}

fn show_pearls(beads: RgbaImage, out_path: &Path, perla: &Path) {
    let perla = image::open(perla).unwrap();
    let (pw, ph) = perla.dimensions();

    RgbaImage::from_par_fn(beads.width() * pw, beads.height() * ph, |x, y| {
        let (ox, px) = (x / pw, x % pw);
        let (oy, py) = (y / pw, y % pw);

        let pc = perla.get_pixel(px, py);
        let c = *beads.get_pixel(ox, oy);

        mul_rgba(c, pc)
    })
    .save(out_path)
    .unwrap();
}

fn mul_rgba(a: Rgba<u8>, b: Rgba<u8>) -> Rgba<u8> {
    Rgba(array::from_fn(|i| {
        ((a.0[i] as u16 * b.0[i] as u16) / 255) as u8
    }))
}
