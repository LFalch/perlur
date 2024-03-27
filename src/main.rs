use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use clap::Parser;
use image::Rgb;
use process::{DistanceMeasure, DownscaleFilter};

use crate::process::{create_beads, output};

mod process;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The image to convert
    input_img: PathBuf,
    #[arg(short, long = "out")]
    /// The resulting file, if no value is given the input path with the extension `.perlur.png` is used
    output_path: Option<PathBuf>,
    /// The amount of pixels squared to read per bead
    #[arg(short, long, default_value = "1")]
    bead_density: u32,
    /// Scale of output picture
    #[arg(short = 's', long)]
    output_scale: Option<u32>,

    #[arg(long = "dist", default_value = "lab")]
    /// The algorithm to calculate distance between palette colours and colours in the source image
    /// used to determine the output colour
    distance: DistanceMeasure,
    #[arg(long = "filter", default_value = "catmull_rom")]
    /// Method with which to downscale the image `BEAD_DENSITY` times
    downscale_filter: DownscaleFilter,

    #[arg(short, long, default_value = "palette.txt")]
    /// Path to palette file formatted as lines of a colour name, a space and then the RGB hex value of the colour
    palette: PathBuf,
    #[arg(long, default_value = "perla.png", conflicts_with("output_scale"))]
    /// If no `OUTPUT_SCALE` is given, this image for each bead multiplying the bead colour
    perla: PathBuf,
}

fn main() {
    let Args {
        input_img,
        output_path,
        bead_density,
        output_scale,
        distance,
        downscale_filter,
        palette,
        perla,
    } = Args::parse();
    let output_path = output_path.unwrap_or_else(|| input_img.with_extension("perlur.png"));

    let palette = read_palette(&palette);

    let (frequency, beads) = create_beads(
        &input_img,
        bead_density,
        &palette,
        distance,
        downscale_filter,
    );

    let mut total_pearls = 0;
    for (name, pearls) in frequency {
        total_pearls += pearls;
        println!("{name}: {pearls}");
    }
    println!(" Total: {total_pearls}");

    output(beads, &output_path, output_scale, &perla);
}

fn read_palette(path: &Path) -> Vec<(Box<str>, Rgb<u8>)> {
    let mut palette = Vec::new();
    for line in BufReader::new(File::open(path).unwrap()).lines() {
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
