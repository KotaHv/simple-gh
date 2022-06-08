from fastapi import APIRouter, Request, HTTPException
from fastapi.responses import PlainTextResponse

from .. import client
from ..config import settings
from ..logger import logger

router = APIRouter()


@router.get("/{github_path:path}", response_class=PlainTextResponse)
async def get_gh(request: Request, github_path: str, token: str | None = None):
    logger.info(f"IP Address: {request.client.host} - request url: {request.url}")
    if token != settings.token:
        logger.error(
            f"IP Address: {request.client.host} - request url: {request.url} - headers:{request.headers}"
        )
        raise HTTPException(status_code=404, detail="not found")
    url = "https://raw.githubusercontent.com/" + github_path
    response = await client.get(url, timeout=5)
    return response.text
