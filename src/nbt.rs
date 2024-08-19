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

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::PathBuf;

use flate2::bufread::{GzDecoder, ZlibDecoder};
use pyo3::types::{PyBytes, PyDict, PyDictMethods, PyTuple};
use pyo3::{pyclass, pymethods, Bound, Py, PyAny, PyResult, Python, ToPyObject};

use crate::{CorruptChunkError, CorruptNBTError, CorruptRegionError, FileSystemError};

/// Reader for the Named Binary Tag format used by Minecraft
pub struct NbtFileReader<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: Read> NbtFileReader<R> {
    pub fn open(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
        }
    }

    fn read(&mut self, len: usize) -> PyResult<&[u8]> {
        if self.buf.len() < len {
            let remaining = len - self.buf.len();
            self.buf.reserve(remaining);
            for _ in 0..remaining {
                self.buf.push(0);
            }
        }

        self.reader
            .read_exact(&mut self.buf[..len])
            .map_err(|e| FileSystemError::new_err(format!("Failed to read file: {:?}", e)))?;

        Ok(&self.buf[..len])
    }

    fn read_end(&mut self) -> u8 {
        0
    }

    fn read_byte(&mut self) -> PyResult<u8> {
        Ok(self.read(1)?[0])
    }

    fn read_short(&mut self) -> PyResult<i16> {
        Ok(i16::from_be_bytes(self.read(2)?.try_into().unwrap()))
    }

    fn read_int(&mut self) -> PyResult<i32> {
        Ok(i32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }

    fn read_long(&mut self) -> PyResult<i64> {
        Ok(i64::from_be_bytes(self.read(8)?.try_into().unwrap()))
    }

    fn read_float(&mut self) -> PyResult<f32> {
        Ok(f32::from_be_bytes(self.read(4)?.try_into().unwrap()))
    }

    fn read_double(&mut self) -> PyResult<f64> {
        Ok(f64::from_be_bytes(self.read(8)?.try_into().unwrap()))
    }

    fn read_byte_array<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
        let len = u32::from_be_bytes(self.read(4)?.try_into().unwrap()) as usize;
        let data = self.read(len)?;
        Ok(PyBytes::new_bound(py, data))
    }

    fn read_int_array<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let len = u32::from_be_bytes(self.read(4)?.try_into().unwrap()) as usize;
        let data = self.read(len * 4)?;
        let values = data
            .chunks_exact(4)
            .map(|d| i32::from_be_bytes(d.try_into().unwrap()));

        Ok(PyTuple::new_bound(py, values))
    }

    fn read_long_array<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let len = u32::from_be_bytes(self.read(4)?.try_into().unwrap()) as usize;
        let data = self.read(len * 8)?;
        let values = data
            .chunks_exact(8)
            .map(|d| i64::from_be_bytes(d.try_into().unwrap()));

        Ok(PyTuple::new_bound(py, values))
    }

    fn read_string(&mut self) -> PyResult<String> {
        let len = u16::from_be_bytes(self.read(2)?.try_into().unwrap()) as usize;
        let data = self.read(len)?;

        Ok(String::from_utf8_lossy(data).to_string())
    }

    fn read_list<'py>(&mut self, py: Python<'py>) -> PyResult<Vec<Py<PyAny>>> {
        let tag_id = self.read_byte()?;
        let len = u32::from_be_bytes(self.read(4)?.try_into().unwrap()) as usize;

        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            let value = match tag_id {
                0 => self.read_end().to_object(py),
                1 => self.read_byte()?.to_object(py),
                2 => self.read_short()?.to_object(py),
                3 => self.read_int()?.to_object(py),
                4 => self.read_long()?.to_object(py),
                5 => self.read_float()?.to_object(py),
                6 => self.read_double()?.to_object(py),
                7 => self.read_byte_array(py)?.to_object(py),
                8 => self.read_string()?.to_object(py),
                9 => self.read_list(py)?.to_object(py),
                10 => self.read_compound(py)?.to_object(py),
                11 => self.read_int_array(py)?.to_object(py),
                12 => self.read_long_array(py)?.to_object(py),
                i => {
                    return Err(CorruptNBTError::new_err(format!(
                        "Invalid list tag id: {}",
                        i
                    )))
                }
            };

            list.push(value);
        }

        Ok(list)
    }

    fn read_compound<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let tags = PyDict::new_bound(py);

        loop {
            let tag_type = self.read(1)?[0];
            if tag_type == 0 {
                break;
            }

            let name = self.read_string()?;

            let payload = match tag_type {
                0 => self.read_end().to_object(py),
                1 => self.read_byte()?.to_object(py),
                2 => self.read_short()?.to_object(py),
                3 => self.read_int()?.to_object(py),
                4 => self.read_long()?.to_object(py),
                5 => self.read_float()?.to_object(py),
                6 => self.read_double()?.to_object(py),
                7 => self.read_byte_array(py)?.to_object(py),
                8 => self.read_string()?.to_object(py),
                9 => self.read_list(py)?.to_object(py),
                10 => self.read_compound(py)?.to_object(py),
                11 => self.read_int_array(py)?.to_object(py),
                12 => self.read_long_array(py)?.to_object(py),
                i => {
                    return Err(CorruptNBTError::new_err(format!(
                        "Invalid list tag id: {}",
                        i
                    )))
                }
            };

            tags.set_item(name, payload).expect("Failed to add to dict");
        }

        Ok(tags)
    }

    /// Reads the entire file and returns (name, payload)
    /// name is the name of the root tag, and payload is a dictionary mapping
    /// names to their payloads
    pub fn read_all<'py>(&mut self, py: Python<'py>) -> PyResult<(String, Bound<'py, PyDict>)> {
        let tag_type = self.read(1)?[0];
        if tag_type != 10 {
            return Err(CorruptNBTError::new_err("Expected a tag compund"));
        }

        let name = self.read_string()?;
        let payload = self.read_compound(py)?;
        return Ok((name, payload));
    }
}

enum RegionData {
    NotLoaded(BufReader<File>),
    Loaded(Vec<u8>),
}

impl RegionData {
    fn load_data(&mut self) -> PyResult<&[u8]> {
        if let Self::NotLoaded(reader) = self {
            let mut data = Vec::new();
            reader.read_to_end(&mut data).map_err(|e| {
                CorruptChunkError::new_err(format!("Failed to read region data: {:?}", e))
            })?;
            *self = RegionData::Loaded(data);
        }

        match self {
            RegionData::Loaded(data) => Ok(data.as_slice()),
            RegionData::NotLoaded(_) => unreachable!(),
        }
    }
}

/// Reader for chunk region files, as introduced in the Beta 1.3 update.
/// It provides functions for opening individual chunks (as (name, data) tuples), getting chunk timestamps,
/// and for listing chunks contained in the file.
/// For reference, the MCR format is outlined at
/// http://www.minecraftwiki.net/wiki/Beta_Level_Format
#[pyclass]
pub struct McrFileReader {
    region_data: RegionData,
    locations: [u32; 1024],
    timestamps: [i32; 1024],
}

#[pymethods]
impl McrFileReader {
    #[new]
    pub fn open(path: PathBuf) -> PyResult<Self> {
        let file = File::open(&path).expect("File does not exist");
        let mut reader = BufReader::new(file);

        let mut location_data = [0; 4096];
        reader.read_exact(&mut location_data).map_err(|e| {
            CorruptRegionError::new_err(format!("Error reading location table: {:?}", e))
        })?;

        let mut locations = [0; 1024];
        for (loc, loc_bytes) in locations.iter_mut().zip(location_data.chunks_exact(4)) {
            *loc = u32::from_be_bytes(loc_bytes.try_into().unwrap());
        }

        let mut timestamp_data = [0; 4096];
        reader.read_exact(&mut timestamp_data).map_err(|e| {
            CorruptRegionError::new_err(format!("Error reading timestamp table: {:?}", e))
        })?;

        let mut timestamps = [0; 1024];
        for (ts, ts_bytes) in timestamps.iter_mut().zip(timestamp_data.chunks_exact(4)) {
            *ts = i32::from_be_bytes(ts_bytes.try_into().unwrap());
        }

        Ok(Self {
            region_data: RegionData::NotLoaded(reader),
            locations,
            timestamps,
        })
    }

    /// List the chunks contained in this region.
    /// To load these chunks, provide these coordinates to [`load_chunk`].
    pub fn get_chunks(&self) -> Vec<(i32, i32)> {
        let mut chunks = Vec::with_capacity(32 * 32);
        for x in 0..32 {
            for z in 0..32 {
                if self.locations[(x + z * 32) as usize] >> 8 != 0 {
                    chunks.push((x, z));
                }
            }
        }

        chunks
    }

    /// Return the given chunk's modification time.
    /// If the given chunk doesn't exist, this number may be nonsense.
    /// Like [`load_chunk`], this will wrap x and z into the range [0, 31].
    pub fn get_chunk_timestamp(&self, x: i32, z: i32) -> i32 {
        self.timestamps[(x.rem_euclid(32) + z.rem_euclid(32) * 32) as usize]
    }

    /// Determine if a chunk exists.
    pub fn chunk_exists(&self, x: i32, z: i32) -> bool {
        self.locations[(x.rem_euclid(32) + z.rem_euclid(32) * 32) as usize] >> 8 != 0
    }

    /// Return a (name, data) tuple for the given chunk, or None if the given chunk doesn't exist in this region file.
    /// If you provide an x or z not between 0 and 31, it will be modulo'd into this range (x % 32, etc).
    /// This is so you can provide chunk coordinates in global coordinates,
    /// and still have the chunks load out of regions properly.
    pub fn load_chunk<'py>(
        &mut self,
        py: Python<'py>,
        x: i32,
        z: i32,
    ) -> PyResult<Option<(String, Bound<'py, PyDict>)>> {
        let location = self.locations[(x.rem_euclid(32) + z.rem_euclid(32) * 32) as usize];
        let offset = (location >> 8) * 4096;

        if offset == 0 {
            return Ok(None);
        }

        let data_offset = offset as usize - 8192; // We already read the header

        let region_data = self.region_data.load_data()?;

        let data_len = u32::from_be_bytes(
            region_data[data_offset..data_offset + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let compression = region_data[data_offset + 4];
        let gzip = match compression {
            // gzip -- not used by the official client, but trivial to
            // support here so...
            1 => true,
            // deflate -- pure zlib stream
            2 => false,
            c => {
                return Err(CorruptRegionError::new_err(format!(
                    "Unsupported compression type: {} (should be 1 or 2)",
                    c
                )))
            }
        };

        if data_offset + data_len + 4 > region_data.len() {
            return Err(CorruptRegionError::new_err("Chunk length is invalid"));
        }

        // Len includes compression byte
        let chunk_data = Cursor::new(&region_data[data_offset + 5..data_offset + 5 + data_len - 1]);

        let data = if gzip {
            Some(
                NbtFileReader::open(GzDecoder::new(chunk_data))
                    .read_all(py)
                    .map_err(|e| {
                        CorruptChunkError::new_err(format!("Count not parse chunk NBT: {:?}", e))
                    })?,
            )
        } else {
            Some(
                NbtFileReader::open(ZlibDecoder::new(chunk_data))
                    .read_all(py)
                    .map_err(|e| {
                        CorruptChunkError::new_err(format!("Count not parse chunk NBT: {:?}", e))
                    })?,
            )
        };

        Ok(data)
    }
}
