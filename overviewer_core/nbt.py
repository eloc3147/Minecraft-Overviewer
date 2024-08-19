#    This file is part of the Minecraft Overviewer.
#
#    Minecraft Overviewer is free software: you can redistribute it and/or
#    modify it under the terms of the GNU General Public License as published
#    by the Free Software Foundation, either version 3 of the License, or (at
#    your option) any later version.
#
#    Minecraft Overviewer is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General
#    Public License for more details.
#
#    You should have received a copy of the GNU General Public License along
#    with the Overviewer.  If not, see <http://www.gnu.org/licenses/>.


class CorruptionError(Exception):
    pass


class CorruptRegionError(CorruptionError):
    """An exception raised when the MCRFileReader class encounters an
    error during region file parsing.
    """
    pass


class CorruptChunkError(CorruptionError):
    pass


class CorruptNBTError(CorruptionError):
    """An exception raised when the NBTFileReader class encounters
    something unexpected in an NBT file."""
    pass
