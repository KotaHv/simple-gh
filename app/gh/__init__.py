from fastapi import APIRouter
from fastapi.responses import PlainTextResponse
from .. import client

router = APIRouter()


@router.get("/{github_path:path}", response_class=PlainTextResponse)
async def get_gh(github_path: str):
    url = "https://raw.githubusercontent.com/" + github_path
    response = await client.get(url, timeout=5)
    return response.text
