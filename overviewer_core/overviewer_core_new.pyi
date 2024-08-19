from typing import Any


class CorruptionError(Exception):
    ...


class FileSystemError(CorruptionError):
    ...


class CorruptRegionError(CorruptionError):
    ...


class CorruptChunkError(CorruptionError):
    ...


class CorruptNBTError(CorruptionError):
    ...


def load(path: str) -> tuple[str, dict[str, Any]]:
    ...


class McrFileReader:
    def __init__(self, path: str) -> None:
        ...

    def get_chunks(self) -> list[tuple[int, int]]:
        """List the chunks contained in this region.
        To load these chunks, provide these coordinates to `load_chunk`.
        """
        ...

    
    def get_chunk_timestamp(self, x: int, z: int) -> int:
        """Return the given chunk's modification time.
        If the given chunk doesn't exist, this number may be nonsense.
        Like `load_chunk`, this will wrap x and z into the range [0, 31].
        """
        ...

    def chunk_exists(self, x: int, z: int) -> bool:
        """Determine if a chunk exists."""
        ...

    
    def load_chunk(self, x: int, z: int) -> tuple[str, dict[str, Any]] | None:
        """Return a (name, data) tuple for the given chunk, or None if the given chunk doesn't exist in this region file.
        If you provide an x or z not between 0 and 31, it will be modulo'd into this range (x % 32, etc).
        This is so you can provide chunk coordinates in global coordinates,
        and still have the chunks load out of regions properly."""
        ...
