use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkError, SinkMatch},
};
use walkdir::WalkDir;

struct CustomSink<'a>(pub &'a RegexMatcher, pub Vec<String>);

struct CustomSinkError;
impl SinkError for CustomSinkError {
    fn error_message<T: std::fmt::Display>(_message: T) -> Self {
        CustomSinkError
    }
}

impl<'a> Sink for CustomSink<'a> {
    type Error = CustomSinkError;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        let range = self.0.find(mat.bytes()).unwrap();
        if let Some(range) = range {
            let line = String::from_utf8_lossy(mat.bytes());
            let mat = line[range].to_string();
            self.1.push(mat);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn join_paths<T: AsRef<Path>, B: AsRef<Path>>(target: T, base: B) -> String {
    let target = PathBuf::from(target.as_ref());

    if target.has_root() {
        return target.display().to_string();
    }

    let mut base = PathBuf::from(base.as_ref());

    for section in target.iter() {
        match section.to_str().unwrap() {
            "." => {
                continue;
            }
            ".." => {
                base.pop();
            }
            str => base.push(str),
        }
    }

    base.display().to_string()
}

struct Flags {
    verbose: bool,
    debug: bool,
}

fn help() {
    println!("Usage: davinci-cleaner <source> <assets> [url-prefix] [flags]");
    println!("");
    println!("Flags:");
    println!("  v: Enable verbose mode.");
    println!("  d: Enable debug mode.");
}

fn main() {
    let mut argv = std::env::args();
    let Some(source) = argv.nth(1) else {
        eprintln!("Error: Needs source argument.\n");
        help();
        return;
    };
    let Some(assets) = argv.next() else {
        eprintln!("Error: Needs assets argument.\n");
        help();
        return;
    };

    let url_prefix = argv.next();

    let flags = argv
        .next()
        .map(|flags| Flags {
            verbose: flags.contains("v"),
            debug: flags.contains("d"),
        })
        .unwrap_or(Flags {
            verbose: false,
            debug: false,
        });

    let cwd = std::env::current_dir().expect("Needs current dir permissions");
    let source = join_paths(source, &cwd);
    let assets = join_paths(assets, &cwd);

    if flags.debug {
        println!("Source: {source}");
        println!("Assets: {assets}");
    }

    if let Err(err) = fs::metadata(&source) {
        eprintln!("Source directory doesn't exists");
        eprintln!("{err}");
        return;
    }

    if let Err(err) = fs::metadata(&assets) {
        eprintln!("Assets directory doesn't exists");
        eprintln!("{err}");
        return;
    }

    let Ok(matcher) = RegexMatcher::new_line_matcher(r#"(/[^/]+)*/[^/]+\.[a-zA-Z0-9]+"#) else {
        eprintln!("Error building regex");
        return;
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
            "\x1b[34m{file}\x1b[0m: \x1b[33m{count}\x1b[0m images",
            file = file_path.display(),
            count = sink.1.len()
        );

        for img in sink.1 {
            if flags.debug {
                println!("[-] \x1b[35m{img}\x1b[0m");
            }

            let img = if let Some(url_prefix) = &url_prefix {
                img.strip_prefix(url_prefix).unwrap_or(&img).to_string()
            } else {
                img
            };

            if flags.debug {
                println!("[--] \x1b[35m{img}\x1b[0m");
            }

            let img = join_paths(&img[1..], &assets);

            if flags.debug {
                let img = img.strip_prefix(&cwd.display().to_string()).unwrap_or(&img);
                println!("[---] \x1b[35m{img}\x1b[0m");
            }
            using_images.insert(img);
        }
    }

    if flags.verbose {
        for img in &using_images {
            let img = img.strip_prefix(&cwd.display().to_string()).unwrap_or(img);

            println!("[Used] \x1b[35m{img}\x1b[0m");
        }
    }

    println!(
        "\nUsed images: \x1b[33m{count}\x1b[0m",
        count = using_images.len()
    );

    for img in WalkDir::new(assets).into_iter().filter_map(Result::ok) {
        if !img.file_type().is_file() {
            continue;
        }

        let img_path = img.path();
        let img = img_path.display().to_string();

        if using_images.contains(&img) {
            continue;
        }

        let img = img_path
            .strip_prefix(&cwd.display().to_string())
            .unwrap_or(img_path)
            .display();

        println!("[Unused] \x1b[32m{img}\x1b[0m");
    }
}
