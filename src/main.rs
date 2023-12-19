mod color;
mod file_completer;
mod file_validator;
mod utils;

use color::*;
use file_completer::FilePathCompleter;
use file_validator::FileValidator;
use inquire::required;

use std::{collections::HashSet, fs, path::PathBuf};

use grep::{
    regex::RegexMatcher,
    searcher::{BinaryDetection, SearcherBuilder},
};
use walkdir::WalkDir;

use crate::utils::{join_paths, CustomSink};

struct Flags {
    verbose: bool,
    debug: bool,
    force: bool,
}

fn main() -> Result<(), ()> {
    load_supported_colors();
    let cwd = std::env::current_dir().expect("Needs current dir permissions");

    let mut argv = std::env::args();
    let flags = argv.nth(1);

    let flags = match flags {
        Some(flags) if flags.contains("help") => {
            help();
            return Ok(());
        }
        Some(flags) if flags.contains("undo") => {
            return undo(cwd);
        }
        Some(flags) => Flags {
            verbose: flags.contains("v"),
            debug: flags.contains("d"),
            force: flags.contains("f"),
        },
        None => Flags {
            verbose: false,
            debug: false,
            force: false,
        },
    };

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

    let Ok(matcher) = RegexMatcher::new_line_matcher(r#"(/[^/]+)*/[^/]+\.(png|jpg|jpeg|webp)"#)
    else {
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
            "{c_blue}{file}{c_reset}: {c_yellow}{count}{c_reset} images",
            file = file_path.display(),
            count = sink.1.len()
        );

        for img in sink.1 {
            if flags.debug {
                println!("[-] {c_magenta}{img}{c_reset}");
            }

            let img = if img.starts_with('/') {
                &img[1..]
            } else {
                &img[..]
            };

            if img.starts_with('*') {
                continue;
            }

            if url_prefix.is_none() {
                println!("Select prefix (cropped section):\n{c_magenta}{img}{c_reset}");
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
                println!("[--] {c_magenta}{img}{c_reset}");
            }

            let img = join_paths(&img[1..], &assets);

            if flags.debug {
                println!("[---] {c_magenta}{img}{c_reset}");
            }
            using_images.insert(img);
        }
    }

    if flags.verbose {
        for img in &using_images {
            let img = img.strip_prefix(&cwd.display().to_string()).unwrap_or(img);

            println!("[Used] {c_magenta}{img}{c_reset}");
        }
    }

    println!("{c_yellow} -------------- {c_reset}",);

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

        println!("[Unused] {c_green}{img}{c_reset}");
    }

    println!(
        "\nUsed images: {c_yellow}{count}{c_reset}",
        count = using_images.len()
    );

    println!(
        "Unused images: {c_yellow}{count}{c_reset}",
        count = unused_images.len()
    );

    let delete_all = inquire::Confirm::new("Delete all files?")
        .with_default(true)
        .prompt()
        .map_err(|err| {
            println!("{err}");
        })?;

    if delete_all && flags.force {
        for img in unused_images {
            let _ = fs::remove_file(&img)
                .map_err(|err| println!("{c_magenta}{img}{c_reset}: {c_red}{err}{c_reset}"));
        }
    } else if delete_all {
        let backup_dir = cwd.clone();
        let backup_dir = join_paths(".davinci-cleaner-backup", backup_dir);
        let _ = fs::create_dir(&backup_dir);

        for img in unused_images {
            let img_relative = img.strip_prefix(&cwd.display().to_string()).unwrap_or(&img);
            let img_relative = if img_relative.starts_with('/') {
                &img_relative[1..]
            } else {
                &img_relative[..]
            };
            let backup_img: PathBuf = join_paths(&img_relative, &backup_dir).into();
            let Ok(backup_dirname) = backup_img.parent().ok_or_else(|| {
                println!("Error getting parent of {}", backup_img.display());
            }) else {
                continue;
            };
            let Ok(_) = fs::create_dir_all(&backup_dirname).map_err(|err| {
                let backup_img = backup_img
                    .strip_prefix(&cwd.display().to_string())
                    .unwrap_or(&backup_img);
                println!(
                    "{c_magenta}{backup_img}{c_reset}: {c_red}{err}{c_reset}",
                    backup_img = backup_img.display()
                )
            }) else {
                continue;
            };
            let _ = fs::rename(&img, &backup_img).map_err(|err| {
                let backup_img = backup_img
                    .strip_prefix(&cwd.display().to_string())
                    .unwrap_or(&backup_img);
                println!(
                    "{c_magenta}{img_relative} -> {backup_img}{c_reset}: {c_red}{err}{c_reset}",
                    backup_img = backup_img.display()
                )
            })?;
        }
    }

    Ok(())
}

fn help() {
    println!("Usage: davinci-cleaner [flags]");
    println!("");
    println!("Flags:");
    println!("  v: Enable verbose mode.");
    println!("  d: Enable debug mode.");
    println!("  f: Force delete. (Backup is made by default)");
}

fn undo(cwd: PathBuf) -> Result<(), ()> {
    let backup_dir = join_paths(".davinci-cleaner-backup", &cwd);
    let mut recovered_images = 0;

    for img in WalkDir::new(&backup_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|f| f.file_type().is_file())
    {
        let backup_img = img.into_path();
        let img_relative = backup_img
            .strip_prefix(&backup_dir)
            .unwrap_or(&backup_img)
            .to_string_lossy();
        let img_relative = if img_relative.starts_with('/') {
            &img_relative[1..]
        } else {
            &img_relative[..]
        };
        println!("{c_magenta}{img_relative}{c_reset}");

        let real_img: PathBuf = join_paths(img_relative, &cwd).into();
        let real_dirname = real_img.parent().ok_or_else(|| {
            println!("Error getting parent of {}", real_img.display());
        })?;

        let _ = fs::create_dir_all(&real_dirname).map_err(|err| {
            let real_img = real_img
                .strip_prefix(&cwd.display().to_string())
                .unwrap_or(&real_img);
            println!(
                "{c_magenta}{img}{c_reset}: {c_red}{err}{c_reset}",
                img = real_img.display()
            )
        });
        let recovered = fs::rename(&backup_img, &real_img)
            .map_err(|err| {
                let backup_img = real_img
                    .strip_prefix(&cwd.display().to_string())
                    .unwrap_or(&real_img);
                println!(
                    "{c_magenta}{img_relative} -> {backup_img}{c_reset}: {c_red}{err}{c_reset}",
                    backup_img = backup_img.display()
                )
            })
            .is_ok();

        if recovered {
            recovered_images += 1;
        }
    }

    let _ = fs::remove_dir_all(&backup_dir).map_err(|err| {
        let backup_img = backup_dir
            .strip_prefix(&cwd.display().to_string())
            .unwrap_or(&backup_dir);
        println!("{c_magenta}{backup_img}{c_reset}: {c_red}{err}{c_reset}",)
    });

    println!("Recovered Images: {c_yellow}{recovered_images}{c_reset}");

    Ok(())
}
