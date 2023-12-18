// https://docs.rs/crate/inquire/0.6.2/source/examples/complex_autocompletion.rs
//

use std::io::ErrorKind;

use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError,
};

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    cwd: String,
    input: String,
    paths: Vec<String>,
    lcp: String,
}

impl FilePathCompleter {
    pub fn new(cwd: String) -> Self {
        Self {
            cwd,
            ..Default::default()
        }
    }

    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = std::path::PathBuf::from(input);

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    self.cwd.clone()
                } else {
                    p.display().to_string()
                }
            })
            .unwrap_or_else(|| self.cwd.clone());

        let scan_dir = {
            let slash = input.find('/').or_else(|| input.find('\\'));

            if let Some(slash) = slash {
                &input[0..slash]
            } else {
                &fallback_parent
            }
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = if path.is_dir() {
                path.to_string_lossy()
            } else {
                idx = idx.saturating_add(1);
                continue;
            };

            let path_str = path_str
                .strip_prefix(&format!("{}{}", self.cwd, std::path::MAIN_SEPARATOR_STR))
                .unwrap_or(&path_str);
            let path_str = path_str.replace('\\', "/");

            if path_str.contains(&self.input) {
                self.paths.push(path_str);
            }

            idx = idx.saturating_add(1);
        }

        self.lcp = self.longest_common_prefix();

        Ok(())
    }

    fn longest_common_prefix(&self) -> String {
        let mut ret: String = String::new();

        let mut sorted = self.paths.clone();
        sorted.sort();
        if sorted.is_empty() {
            return ret;
        }

        let mut first_word = sorted.first().unwrap().chars();
        let mut last_word = sorted.last().unwrap().chars();

        loop {
            match (first_word.next(), last_word.next()) {
                (Some(c1), Some(c2)) if c1 == c2 => {
                    ret.push(c1);
                }
                _ => return ret,
            }
        }
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        Ok(self.paths.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => match self.lcp.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.lcp.clone()),
            },
        })
    }
}
