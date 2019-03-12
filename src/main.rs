#[macro_use]
extern crate structopt;
#[macro_use]
extern crate clap;

use colored::*;
use glob::glob;
use glob::Paths;
use image::imageops::{overlay, FilterType};
use image::*;

use std::fmt;
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;

/// Describes filetype
arg_enum! {
    #[derive(Debug)]
    enum FileType {
        Png,
    }
}

fn get_ext(ft: &Option<FileType>) -> &'static str {
    match ft {
        Some(t) => match t {
            FileType::Png => "png",
        },
        None => "png",
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Directory from which to get all background images
    #[structopt(short = "b", long = "background-directory", parse(from_os_str))]
    background: PathBuf,

    /// Directory from which to get all foreground images
    #[structopt(short = "f", long = "foreground-directory", parse(from_os_str))]
    foreground: PathBuf,

    /// Directory to which to save all completed images
    #[structopt(short = "o", long = "output-directory", parse(from_os_str))]
    output: PathBuf,

    /// Filetype to use for both background and foreground images
    #[structopt(raw(possible_values = "&FileType::variants()", case_insensitive = "true"))]
    file_type: Option<FileType>,

    /// Size, in pixels, of the completed image. Defaults to the individual background image size.
    #[structopt(short = "s", long = "size")]
    size: Option<usize>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    // Validate directories and print useful information
    let results: Vec<(PathBuf, bool)> = vec![
        (opt.background.clone(), opt.background.exists()),
        (opt.foreground.clone(), opt.foreground.exists()),
        (opt.output.clone(), opt.output.exists()),
    ]; // background, foreground, output dir existence

    for (path, exists) in results {
        if exists {
            println!("{}: {:?} is a directory.", "OK".bold().green(), path);
        } else {
            println!("{}: {:?} is not a directory.", "ERR".bold().red(), path);
            panic!(format!(
                "Please create a directory at {:?}, or pick an existing directory.",
                path
            ));
        }
    }

    // find every image in the bg and fg dirs
    let foregrounds: Paths = glob(&format!(
        "{}/**/*.{}",
        opt.foreground
            .clone()
            .into_os_string()
            .into_string()
            .unwrap(),
        get_ext(&opt.file_type)
    ))
    .unwrap();

    for bg_path in glob(&format!(
        "{}/**/*.{}",
        opt.background
            .clone()
            .into_os_string()
            .into_string()
            .unwrap(),
        get_ext(&opt.file_type)
    ))
    .unwrap()
    {
        let bg_path = bg_path.unwrap();
        let bg_filename = bg_path.file_stem();
        let bg_img: DynamicImage = image::open(bg_path.clone()).expect("Failed reading file");
        let dimensions = bg_img.dimensions();

        for fg_path in glob(&format!(
            "{}/**/*.{}",
            opt.foreground
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
            get_ext(&opt.file_type)
        ))
        .unwrap()
        {
            let fg_path = fg_path.unwrap();
            let fg_filename = fg_path.file_stem();
            // start creating images
            let fg_img: DynamicImage = image::open(fg_path.clone()).expect("Failed reading file");

            // create a new image
            let mut new_img = bg_img.clone();

            // composite images
            let fg_dimensions = fg_img.dimensions();
            let x_offset = (dimensions.0 / 2) - (fg_dimensions.0 / 2);
            let y_offset = (dimensions.1 / 2) - (fg_dimensions.1 / 2);
            overlay(&mut new_img, &fg_img, x_offset, y_offset);

            // save our image
            let out_path = format!(
                "{}/{}-{}.png",
                opt.output.clone().into_os_string().into_string().unwrap(),
                bg_filename.clone().unwrap().to_str().unwrap(),
                fg_filename.clone().unwrap().to_str().unwrap(),
            );
            new_img.save(out_path).expect("Problem saving file.");
        }
    }
}
