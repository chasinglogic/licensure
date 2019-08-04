use std::fs::File;
use std::io;
use std::io::prelude::*;

use crate::config::Config;

pub struct Licensure {
    config: Config,
}

impl Licensure {
    pub fn new(config: Config) -> Licensure {
        Licensure { config }
    }

    pub fn license_files(self, files: &[String]) -> Result<(), io::Error> {
        for file in files {
            if self.config.excludes.is_match(file) {
                continue;
            }

            let templ = match self.config.licenses.get_template(file) {
                Some(t) => t,
                None => {
                    info!("skipping {} because no license config matched.", file);
                    continue;
                }
            };

            let uncommented = templ.render();
            let (cfg, commenter) = self.config.comments.get_commenter(file);
            let mut header = commenter.comment(&uncommented, cfg.get_columns());
            let mut content = String::new();
            {
                let mut f = File::open(file)?;
                f.read_to_string(&mut content)?;
            }

            // TODO: make this smarter about updating years etc.
            if content.contains(&header) {
                info!("{} already licensed", file);
                continue;
            }

            header.push_str(&content);

            if self.config.change_in_place {
                let mut f = File::create(file)?;
                f.write_all(header.as_bytes())?;
            } else {
                println!("{}", header);
            }
        }

        Ok(())
    }
}
