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

    pub fn license_files(self, files: &[String]) -> Result<LicenseStats, io::Error> {
        let mut stats = LicenseStats::new();

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

            let (cfg, commenter) = self.config.comments.get_commenter(file);

            let uncommented = templ.render();
            let mut header = commenter.comment(&uncommented, cfg.get_columns());

            let mut content = String::new();
            {
                let mut f = File::open(file)?;
                f.read_to_string(&mut content)?;
            }

            if content.contains(&header) {
                info!("{} already licensed", file);
                continue;
            }

            let outdated_re = templ.outdated_license_pattern(&commenter, cfg.get_columns());
            if outdated_re.is_match(&content) {
                info!("{} licensed, but year is outdated", file);
                stats.files_needing_license_update.push(file.clone());

                let updated = outdated_re.replace(&content, header);

                if self.config.change_in_place {
                    let mut f = File::create(file)?;
                    f.write_all(updated.as_bytes())?;
                } else {
                    println!("{}", updated);
                }

                continue;
            }

            let trimmed_outdated_re = templ.outdated_license_trimmed_pattern(&commenter, cfg.get_columns());
            if trimmed_outdated_re.is_match(&content) {
                info!("{} licensed, but year is outdated", file);
                stats.files_needing_license_update.push(file.clone());

                let updated = trimmed_outdated_re.replace(&content, header);

                if self.config.change_in_place {
                    let mut f = File::create(file)?;
                    f.write_all(updated.as_bytes())?;
                } else {
                    println!("{}", updated);
                }

                continue;
            }

            stats.files_not_licensed.push(file.clone());

            // if already licensed but the trailing lines/whitespace do not match
            let content_trimmed = content.trim_end();
            let header_trimmed = header.trim_end();

            if content_trimmed.contains(header_trimmed) {
                info!(
                    "{} already licensed but the trailing lines/whitespace do not match",
                    file
                );

                header = content.replace(header_trimmed, &header);
            } else {
                header.push_str(&content);
            }

            if self.config.change_in_place {
                let mut f = File::create(file)?;
                f.write_all(header.as_bytes())?;
            } else {
                println!("{}", header);
            }
        }

        Ok(stats)
    }
}

pub struct LicenseStats {
    pub files_not_licensed: Vec<String>,
    pub files_needing_license_update: Vec<String>,
}

impl LicenseStats {
    fn new() -> Self {
        Self { files_not_licensed: Vec::new(), files_needing_license_update: Vec::new() }
    }
}
