/*
    This file is part of Infinite Escape Velocity.

    Infinite Escape Velocity is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Infinite Escape Velocity is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Infinite Escape Velocity.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::path::{Path, PathBuf};

use bytes::Bytes;
use tokio::task::spawn_blocking;

use super::archive_reader::ArchiveReader;

pub struct FilesystemReader {
    file: PathBuf,
}

impl FilesystemReader {
    pub async fn new(path: &Path) -> Result<Self, ()> {
        match tokio::fs::canonicalize(path).await {
            Ok(exists) => Ok(Self { file: exists }),
            Err(error) => {
                tracing::error!(
                    "Error canonicalizing path for asset folder {} with error {}",
                    path.to_string_lossy(),
                    error.to_string()
                );
                Err(())
            }
        }
    }
}

impl ArchiveReader for FilesystemReader {
    async fn get_directories(&self) -> Vec<PathBuf> {
        let base_path = self.file.clone();
        let directories: Vec<_> = spawn_blocking(|| {
            walkdir::WalkDir::new(base_path)
                .into_iter()
                .filter_map(|dir| match dir {
                    Ok(path) => match path.file_type().is_dir() {
                        true => Some(path.path().join("")),
                        false => None,
                    },
                    Err(error) => {
                        tracing::error!("Error walking directory entry {}", error);
                        None
                    }
                })
                .collect()
        })
        .await
        .unwrap();
        directories
    }

    async fn get_files(&self) -> Vec<PathBuf> {
        let base_path = self.file.clone();
        let files: Vec<_> = spawn_blocking(|| {
            walkdir::WalkDir::new(base_path)
                .into_iter()
                .filter_map(|dir| match dir {
                    Ok(path) => match path.file_type().is_file() {
                        true => Some(path.into_path()),
                        false => None,
                    },
                    Err(error) => {
                        tracing::error!("Error walking directory entry {}", error);
                        None
                    }
                })
                .collect()
        })
        .await
        .unwrap();
        files
    }

    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, ()> {
        let search_path = self.file.join(PathBuf::from(name));
        let canon_path = match tokio::fs::canonicalize(&search_path).await {
            Ok(canon) => canon,
            Err(_error) => {
                // Path does not exist
                return Ok(None);
            }
        };

        if canon_path.starts_with(&self.file) == false {
            tracing::error!(
                "Directory traversal forbidden for path {}",
                canon_path.to_string_lossy()
            );
            return Err(());
        }

        match tokio::fs::read(canon_path).await {
            Ok(read) => Ok(Some(read.into())),
            Err(error) => {
                tracing::error!("Error reading asset file from directory: {}", error);
                Err(())
            }
        }
    }
}
