use std::path::{Path, PathBuf};

use grep::{
    matcher::Matcher,
    regex::RegexMatcher,
    searcher::{Searcher, Sink, SinkError, SinkMatch},
};

pub struct CustomSink<'a>(pub &'a RegexMatcher, pub Vec<String>);

pub struct CustomSinkError;
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

pub fn join_paths<T: AsRef<Path>, B: AsRef<Path>>(target: T, base: B) -> String {
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
