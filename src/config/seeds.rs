use std::{fs, path::Path};

pub trait SeedsConfig {
    fn get_seeds() -> Vec<Vec<u8>>;
}

#[allow(unused)]
pub struct NoSeedsConfig;

impl SeedsConfig for NoSeedsConfig {
    fn get_seeds() -> Vec<Vec<u8>> {
        vec![]
    }
}

#[allow(unused)]
pub struct ValidCorpusSeedsConfig;

impl SeedsConfig for ValidCorpusSeedsConfig {
    fn get_seeds() -> Vec<Vec<u8>> {
        let corpus_dir = Path::new("valid_corpus");
        let mut inputs = vec![];
        match fs::read_dir(corpus_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(bytes) = fs::read(&path) {
                            inputs.push(bytes);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                let current_directory_listing = fs::read_dir(".").unwrap();
                for entry in current_directory_listing.flatten() {
                    eprintln!("{}", entry.path().display());
                }
                panic!("No seeds found in corpus directory");
            }
        }

        assert!(!inputs.is_empty(), "No seeds found in corpus directory");
        inputs
    }
}
