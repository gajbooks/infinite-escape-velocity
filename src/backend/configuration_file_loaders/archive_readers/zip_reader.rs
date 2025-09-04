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

use async_zip::tokio::read::fs::ZipFileReader;
use bytes::Bytes;
use futures_util::AsyncReadExt;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::archive_reader::ArchiveReader;

pub struct ZipReader {
    file: ZipFileReader,
    index: HashMap<String, usize>,
}

impl ZipReader {
    pub async fn new(path: &Path) -> Result<Self, ()> {
        match ZipFileReader::new(path).await {
            Ok(file) => {
                let index = file
                    .file()
                    .entries()
                    .iter()
                    .enumerate()
                    .filter_map(
                        |(index, zip_contents)| match zip_contents.filename().as_str() {
                            Ok(valid) => Some((valid.to_string(), index)),
                            Err(not_utf8) => {
                                tracing::error!(
                                    "File name {} is not UTF 8 in asset bundle zip {}",
                                    not_utf8.to_string(),
                                    path.to_string_lossy()
                                );
                                None
                            }
                        },
                    )
                    .collect();
                Ok(Self { file: file, index })
            }
            Err(error) => {
                tracing::error!(
                    "Error reading zip data from asset zip {} with error {}",
                    path.to_string_lossy(),
                    error.to_string()
                );
                Err(())
            }
        }
    }
}

fn zip_filename_is_directory(name: &str) -> bool {
    // Verbatim the logic used to determine if a file is a directory per async-zip crate
    name.ends_with('/')
}

impl ArchiveReader for ZipReader {
    async fn get_directories(&self) -> Vec<PathBuf> {
        self.index
            .iter()
            .filter(|(filename, _index)| zip_filename_is_directory(filename))
            .map(|(filename, _index)| filename.into())
            .collect()
    }

    async fn get_files(&self) -> Vec<PathBuf> {
        self.index
            .iter()
            .filter(|(filename, _index)| !zip_filename_is_directory(filename))
            .map(|(filename, _index)| filename.into())
            .collect()
    }

    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, ()> {
        // We don't have real paths in Zip land
        let name = name.to_string_lossy();
        match self.index.get(&*name) {
            Some(has) => {
                match self.file.reader_with_entry(*has).await {
                    Ok(mut has_entry) => {
                        let mut buf =
                            Vec::with_capacity(has_entry.entry().uncompressed_size() as usize);
                        match has_entry.read_to_end(&mut buf).await {
                            Ok(_completed) => Ok(Some(buf.into())),
                            Err(read_error) => {
                                // File read has failed
                                tracing::error!(
                                    "Error reading file {} from zip asset bundle: {}",
                                    name,
                                    read_error
                                );
                                Err(())
                            }
                        }
                    }
                    Err(no_reader) => {
                        // Something interesting has happened between filename and index association and an attempted read
                        tracing::error!(
                            "Invalid zip index for asset bundle {} with error {}",
                            self.file.path().to_string_lossy(),
                            no_reader.to_string()
                        );
                        Err(())
                    }
                }
            }
            // Zip simply does not have this file, not an error condition
            None => Ok(None),
        }
    }
}
