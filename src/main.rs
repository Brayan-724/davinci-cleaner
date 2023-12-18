mod color;
mod file_completer;
mod file_validator;
mod utils;

use color::*;
use file_completer::FilePathCompleter;
use file_validator::FileValidator;
use inquire::required;

use std::{collections::HashSet, fs};

use grep::{
    regex::RegexMatcher,
    searcher::{BinaryDetection, SearcherBuilder},
};
use walkdir::WalkDir;

use crate::utils::{join_paths, CustomSink};

struct Flags {
    verbose: bool,
    debug: bool,
}

fn help() {
    println!("Usage: davinci-cleaner [flags]");
    println!("");
    println!("Flags:");
    println!("  v: Enable verbose mode.");
    println!("  d: Enable debug mode.");
}

fn main() -> Result<(), ()> {
    load_supported_colors();
    let cwd = std::env::current_dir().expect("Needs current dir permissions");

    let source = inquire::Text::new("Source Folder: ")
        .with_autocomplete(FilePathCompleter::new(cwd.display().to_string()))
        .with_validator(required!())
        .with_validator(FileValidator::new(cwd.display().to_string()))
        .prompt()
        .map_err(|err| {
            println!("Error prompting source folder");
            println!("{err}");
            ()
        })?;

    let assets = inquire::Text::new("Assets Folder: ")
        .with_autocomplete(FilePathCompleter::new(cwd.display().to_string()))
        .with_validator(required!())
        .with_validator(FileValidator::new(cwd.display().to_string()))
        .prompt()
        .map_err(|err| {
            println!("Error prompting assets folder");
            println!("{err}");
            ()
        })?;

    let mut url_prefix = inquire::Text::new("Url Prefix: ")
        .prompt()
        .ok()
        .filter(|opt| !opt.is_empty());

    let mut argv = std::env::args();
    let flags = argv
        .nth(1)
        .map(|flags| Flags {
            verbose: flags.contains("v"),
            debug: flags.contains("d"),
        })
        .unwrap_or(Flags {
            verbose: false,
            debug: false,
        });

    let source = join_paths(source, &cwd);
    let assets = join_paths(assets, &cwd);

    if flags.debug {
        println!("Source: {source}");
        println!("Assets: {assets}");
    }

    if let Err(err) = fs::metadata(&source) {
        eprintln!("Source directory doesn't exists");
        eprintln!("{err}");
        return Ok(());
    }

    if let Err(err) = fs::metadata(&assets) {
        eprintln!("Assets directory doesn't exists");
        eprintln!("{err}");
        return Ok(());
    }

    let Ok(matcher) = RegexMatcher::new_line_matcher(r#"(/[^/]+)*/[^/]+\.(png|jpg|jpeg|webp)"#) else {
        eprintln!("Error building regex");
        return Ok(());
    };
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();

    let mut using_images = HashSet::new();

    for result in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        if !result.file_type().is_file() {
            continue;
        }

        let file_path = result.path();

        if !matches!(
            file_path.extension().map(|s| s.to_str().unwrap()),
            Some("html") | Some("js") | Some("css") | Some("xml")
        ) {
            continue;
        }

        let mut sink = CustomSink(&matcher, Vec::new());
        if let Err(_) = searcher.search_path(&matcher, file_path, &mut sink) {
            eprintln!("ERROR: Matching");
        }

        println!(
            "{color_blue}{file}{color_reset}: {color_yellow}{count}{color_reset} images",
            file = file_path.display(),
            count = sink.1.len()
        );

        for img in sink.1 {
            if flags.debug {
                println!("[-] {color_magenta}{img}{color_reset}");
            }

            let img = if img.starts_with('/') {
                &img[1..]
            } else {
                &img[..]
            };

            if url_prefix.is_none() {
                println!("Select prefix (cropped section):\n{color_magenta}{img}{color_reset}");
                url_prefix = inquire::Text::new("Url Prefix")
                    .prompt()
                    .ok()
                    .filter(|opt| !opt.is_empty());
            }

            let img = if let Some(url_prefix) = &url_prefix {
                let url_prefix = if url_prefix.starts_with('/') {
                    &url_prefix[1..]
                } else {
                    &url_prefix[..]
                };

                img.strip_prefix(url_prefix).unwrap_or(&img)
            } else {
                img
            };

            if flags.debug {
                println!("[--] {color_magenta}{img}{color_reset}");
            }

            let img = join_paths(&img[1..], &assets);

            if flags.debug {
                println!("[---] {color_magenta}{img}{color_reset}");
            }
            using_images.insert(img);
        }
    }

    if flags.verbose {
        for img in &using_images {
            let img = img.strip_prefix(&cwd.display().to_string()).unwrap_or(img);

            println!("[Used] {color_magenta}{img}{color_reset}");
        }
    }

    println!("{color_yellow} -------------- {color_reset}",);

    let mut unused_images = HashSet::new();

    for img in WalkDir::new(assets).into_iter().filter_map(Result::ok) {
        if !img.file_type().is_file() {
            continue;
        }

        let img_path = img.path();
        let img = img_path.display().to_string();

        if using_images.contains(&img) {
            continue;
        }

        unused_images.insert(img);

        let img = img_path
            .strip_prefix(&cwd.display().to_string())
            .unwrap_or(img_path)
            .display();

        println!("[Unused] {color_green}{img}{color_reset}");
    }

    println!(
        "\nUsed images: {color_yellow}{count}{color_reset}",
        count = using_images.len()
    );

    println!(
        "Unused images: {color_yellow}{count}{color_reset}",
        count = unused_images.len()
    );

    Ok(())
}
