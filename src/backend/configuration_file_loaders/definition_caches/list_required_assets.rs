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

/*
Trait requiring the implementation of a method to list all the assets that a definition requires.
Servers may not always serve assets or keep them loaded,
    but the required assets and their types needs to be known to generate string->ID dictionaries for lower network overhead,
    as well as other tasks like optionally verifying if a server does in fact have all the required resources loaded for development and debugging purposes.
 */

use crate::configuration_file_structures::{asset_definition_file::AssetType, reference_string_types::AssetReference};

pub trait ListRequiredAssets {
    fn get_required_asset_list(&self) -> Vec<(&AssetReference, AssetType)>;
}