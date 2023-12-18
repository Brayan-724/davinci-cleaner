use std::{io, path::PathBuf};

use inquire::{
    validator::{StringValidator, Validation},
    CustomUserError,
};

use crate::utils::join_paths;

#[derive(Clone)]
pub struct FileValidator {
    cwd: String,
}

impl FileValidator {
    pub fn new(cwd: String) -> Self {
        Self { cwd }
    }
}

impl StringValidator for FileValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        let path = join_paths(input, &self.cwd);
        let path: PathBuf = path.into();

        let meta = match path.metadata() {
            Ok(m) => m,
            Err(err) => {
                return match err.kind() {
                    io::ErrorKind::NotFound => Ok(Validation::Invalid(
                        "Cannot find specified folder. NotFound".into(),
                    )),
                    io::ErrorKind::PermissionDenied => Ok(Validation::Invalid(
                        "Cannot access to specified folder. PermissionDenied".into(),
                    )),
                    _ => Ok(Validation::Invalid(format!("{err}").into())),
                }
            }
        };

        if !meta.is_dir() {
            return Ok(Validation::Invalid("Specified path is not a folder".into()));
        }

        Ok(Validation::Valid)
    }
}
