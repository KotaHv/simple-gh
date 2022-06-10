import time
from pathlib import Path

import aiofiles
from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import Response

from .. import client
from ..config import settings
from ..logger import logger

router = APIRouter()
cache_dir = Path(settings.cache_dir)
cache_dir.mkdir(parents=True, exist_ok=True)


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
    try:
        stat = filepath.stat()
        if time.time() - stat.st_ctime <= settings.cache_time:
            async with aiofiles.open(filepath, "rb") as f:
                return Response(content=await f.read())
        logger.info(f"{github_path} cache has expired")
    except FileNotFoundError:
        pass
    url = "https://raw.githubusercontent.com/" + github_path

    response = await client.get(url, timeout=5)
    async with aiofiles.open(filepath, "wb") as f:
        await f.write(response.content)
    return Response(content=response.content)
