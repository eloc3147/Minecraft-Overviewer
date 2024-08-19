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

use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::PathBuf;

use flate2::bufread::{GzDecoder, ZlibDecoder};
use pyo3::types::{PyDict, PyModule, PyModuleMethods};
use pyo3::{pyfunction, pymodule, wrap_pyfunction, Bound, PyResult, Python};

use nbt::{McrFileReader, NbtFileReader};

#[pyfunction]
/// Reads in the given file as NBT format, parses it, and returns the result as a (name, data) tuple.
fn load<'py>(path: PathBuf, py: Python<'py>) -> (String, Bound<'py, PyDict>) {
    let file = File::open(&path).expect("File does not exist");
    let gzip = GzDecoder::new(BufReader::new(file));
    let mut reader = NbtFileReader::open(gzip);

    reader.read_all(py)
}

// @_file_loader
// def load_region(fileobj):
//     """Reads in the given file as a MCR region, and returns an object
//     for accessing the chunks inside."""
//     return MCRFileReader(fileobj)
//
//
// class CorruptionError(Exception):
//     pass
//
//
// class CorruptRegionError(CorruptionError):
//     """An exception raised when the MCRFileReader class encounters an
//     error during region file parsing.
//     """
//     pass
//
//
// class CorruptChunkError(CorruptionError):
//     pass
//
//
// class CorruptNBTError(CorruptionError):
//     """An exception raised when the NBTFileReader class encounters
//     something unexpected in an NBT file."""
//     pass

//
//     def __init__(self, fileobj, is_gzip=True):
//         """Create a NBT parsing object with the given file-like
//         object. Setting is_gzip to False parses the file as a zlib
//         stream instead."""
//         if is_gzip:
//             self._file = gzip.GzipFile(fileobj=fileobj, mode='rb')
//         else:
//             # pure zlib stream -- maybe later replace this with
//             # a custom zlib file object?
//             data = zlib.decompress(fileobj.read())
//             self._file = BytesIO(data)

#[pymodule]
fn overviewer_core_new(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?)?;

    m.add_class::<McrFileReader>()?;
    Ok(())
}
