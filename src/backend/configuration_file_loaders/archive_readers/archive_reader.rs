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

pub trait ArchiveReader {
    // Even though some implementations use pure Strings (Zip), we need the Path component parsing for normalizing inconsistent directories for the filesystem implementation

    // Returns a list of all directories in the structure
    async fn get_directories(&self) -> Vec<PathBuf>;

    // Returns a list of all non-directory files in the structure
    async fn get_files(&self) -> Vec<PathBuf>;

    // Attempts to retrieve the file data for a given file
    async fn try_get_file(&self, name: &Path) -> Result<Option<Bytes>, ()>;
}
