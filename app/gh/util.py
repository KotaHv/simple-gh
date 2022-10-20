from enum import Enum, auto

from anyio import Path


async def get_dir_size(dirpath: Path) -> int:
    dir_size = 0
    async for file in dirpath.iterdir():
        if not await file.is_file():
            continue
        if file.suffix != ".type":
            dir_size += (await file.stat()).st_size
    return dir_size


class FileSort(Enum):
    CTIME = auto()
    SIZE = auto()


async def get_dir_files(dirpath: Path,
                        sort: FileSort | None = None) -> list[Path]:
    if sort is None:
        return [
            file async for file in dirpath.iterdir() if
            file.suffix != ".type" and await file.is_file()
        ]
    files = []
    async for file in dirpath.iterdir():
        if file.suffix != ".type" and await file.is_file():
            files.append((file, await file.stat()))
    if sort == FileSort.CTIME:
        files.sort(key=lambda file_tuple: file_tuple[1].st_ctime)
    elif sort == FileSort.SIZE:
        files.sort(key=lambda file_tuple: file_tuple[1].st_size)
    return [file_tuple[0] for file_tuple in files]


async def rm_file(file: Path):
    await file.unlink()
    try:
        await file.with_suffix(file.suffix + ".type").unlink()
    except FileNotFoundError:
        pass
