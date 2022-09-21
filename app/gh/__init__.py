import time

import httpx
import aiofiles
from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import Response

from ..config import settings
from ..logger import logger

router = APIRouter()
cache_dir = settings.cache_dir
cache_dir.mkdir(parents=True, exist_ok=True)
client = httpx.AsyncClient(base_url="https://raw.githubusercontent.com", timeout=5)


def get_cache_size():
    cache_size = 0
    for file in cache_dir.iterdir():
        if file.suffix != ".type":
            cache_size += file.stat().st_size
    return cache_size


cache_size = get_cache_size()


def get_cache_files(sort="st_ctime"):
    files = [file for file in cache_dir.iterdir() if file.suffix != ".type"]
    if sort == "st_ctime":
        files.sort(key=lambda file: file.stat().st_ctime)
    elif sort == "st_size":
        files.sort(key=lambda file: file.stat().st_size)
    return files


def rm_cache_file(file):
    file.unlink()
    file.with_suffix(file.suffix + ".type").unlink()
    logger.info(f"{file} has been deleted")


def rm_cache_files(filepath=None):
    def cache_files(sort="st_ctime"):
        files = get_cache_files(sort=sort)
        if filepath:
            try:
                files.remove(filepath)
            except ValueError:
                pass
        return files

    global cache_size

    files = cache_files()
    for file in files:
        if time.time() - file.stat().st_ctime <= settings.cache_time:
            break
        cache_size -= file.stat().st_size
        rm_cache_file(file)
        if cache_size <= settings.max_cache:
            return
    files = cache_files(sort="st_size")
    for file in files:
        cache_size -= file.stat().st_size
        rm_cache_file(file)
        if cache_size <= settings.max_cache:
            return


@router.get("/{github_path:path}")
async def get_gh(request: Request, github_path: str, token: str | None = None):
    logger.info(
        f"IP Address: {request.headers.get('X-Forwarded-For',request.client.host)} - request url: {request.url}"
    )
    if token != settings.token:
        logger.error(
            f"IP Address: {request.headers.get('X-Forwarded-For',request.client.host)} - request url: {request.url} - headers:{request.headers}"
        )
        raise HTTPException(status_code=404, detail="not found")

    filepath = cache_dir / github_path.replace("/", "_")
    typepath = filepath.with_suffix(filepath.suffix + ".type")

    headers = {}
    content = b""

    global cache_size
    try:
        stat = filepath.stat()
        if time.time() - stat.st_ctime <= settings.cache_time:
            async with aiofiles.open(filepath, "rb") as f:
                content = await f.read()
            async with aiofiles.open(typepath, "r") as f:
                headers["content-type"] = await f.read()
        else:
            cache_size -= stat.st_size
            logger.info(f"{github_path} cache has expired")
            raise FileNotFoundError("cache has expired")
    except FileNotFoundError:
        response = await client.get(github_path)
        file_size = (
            int(response.headers["content-length"])
            if response.headers.get("content-encoding") != "gzip"
            else len(response.content)
        )
        if file_size <= settings.file_max:
            cache_size += file_size
            if cache_size > settings.max_cache:
                rm_cache_files(filepath=filepath)
            async with aiofiles.open(filepath, "wb") as f:
                await f.write(response.content)
            async with aiofiles.open(typepath, "w") as f:
                await f.write(response.headers["content-type"])
        content = response.content
        headers["content-type"] = response.headers["content-type"]
    return Response(content=content, headers=headers)
