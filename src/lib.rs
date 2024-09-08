//    This file is part of the Minecraft Overviewer.
//
//    Minecraft Overviewer is free software: you can redistribute it and/or
//    modify it under the terms of the GNU General Public License as published
//    by the Free Software Foundation, either version 3 of the License, or (at
//    your option) any later version.
//
//    Minecraft Overviewer is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General
//    Public License for more details.
//
//    You should have received a copy of the GNU General Public License along
//    with the Overviewer.  If not, see <http://www.gnu.org/licenses/>.

mod nbt;
mod texture;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use flate2::bufread::GzDecoder;
use pyo3::exceptions::PyException;
use pyo3::types::{PyDict, PyModule, PyModuleMethods};
use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, PyResult, Python};

use nbt::{McrFileReader, NbtFileReader};
use texture::transform_image_side;

pyo3::create_exception!(overviewer_core_new, CorruptionError, PyException);
pyo3::create_exception!(overviewer_core_new, FileSystemError, CorruptionError);
pyo3::create_exception!(overviewer_core_new, CorruptRegionError, CorruptionError);
pyo3::create_exception!(overviewer_core_new, CorruptChunkError, CorruptionError);
pyo3::create_exception!(overviewer_core_new, CorruptNBTError, CorruptionError);

#[pyfunction]
/// Reads in the given file as NBT format, parses it, and returns the result as a (name, data) tuple.
fn load<'py>(path: PathBuf, py: Python<'py>) -> PyResult<(String, Bound<'py, PyDict>)> {
    let file = File::open(&path)
        .map_err(|e| FileSystemError::new_err(format!("Error opening file: {:?}", e)))?;
    let gzip = GzDecoder::new(BufReader::new(file));
    let mut reader = NbtFileReader::open(gzip);

    reader.read_all(py)
}

#[pymodule]
fn overviewer_core_new(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add(
        "CorruptionError",
        m.py().get_type_bound::<CorruptionError>(),
    )?;
    m.add(
        "FileSystemError",
        m.py().get_type_bound::<FileSystemError>(),
    )?;
    m.add(
        "CorruptRegionError",
        m.py().get_type_bound::<CorruptRegionError>(),
    )?;
    m.add(
        "CorruptChunkError",
        m.py().get_type_bound::<CorruptChunkError>(),
    )?;
    m.add(
        "CorruptNBTError",
        m.py().get_type_bound::<CorruptNBTError>(),
    )?;

    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_class::<McrFileReader>()?;

    m.add_function(wrap_pyfunction!(transform_image_side, m)?)?;

    Ok(())
}
