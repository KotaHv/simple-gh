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


def get_cache_size():
    cache_size = 0
    for file in cache_dir.iterdir():
        cache_size += file.stat().st_size
    return cache_size


cache_size = get_cache_size()
client = httpx.AsyncClient(base_url="https://raw.githubusercontent.com", timeout=5)


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
    global cache_size
    try:
        stat = filepath.stat()
        if time.time() - stat.st_ctime <= settings.cache_time:
            async with aiofiles.open(filepath, "rb") as f:
                return Response(content=await f.read())
        cache_size -= filepath.stat().st_size
        logger.info(f"{github_path} cache has expired")
    except FileNotFoundError:
        pass

    response = await client.get(github_path)
    file_size = (
        int(response.headers["content-length"])
        if response.headers.get("content-encoding") != "gzip"
        else len(response.content)
    )
    if file_size > settings.file_max:
        return Response(content=response.content)
    cache_size += file_size
    if cache_size > settings.max_cache:
        files = [file for file in cache_dir.iterdir() if file != filepath]
        files.sort(key=lambda file: file.stat().st_ctime)
        for file in files:
            cache_size -= file.stat().st_size
            file.unlink()
            logger.info(f"{file} has been deleted")
            if cache_size <= settings.max_cache:
                break
    async with aiofiles.open(filepath, "wb") as f:
        await f.write(response.content)
    return Response(content=response.content)
