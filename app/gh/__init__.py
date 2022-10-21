import httpx
from fastapi import APIRouter, HTTPException, Depends
from fastapi.responses import FileResponse, Response
from anyio import Path

from ..config import settings

router = APIRouter()
cache_dir = Path(settings.cache_dir)
client = httpx.AsyncClient(base_url="https://raw.githubusercontent.com",
                           timeout=5)


def token_guard(token: str | None = None):
    if settings.token:
        if token != settings.token:
            raise HTTPException(status_code=404)


def path_guard(github_path: str):
    if len(github_path.replace("/", "")) == 0:
        raise HTTPException(status_code=500)
    return github_path


async def write_file(filepath: Path, content: bytes, typepath: Path,
                     headers: dict):
    await typepath.write_text(headers["content-type"], encoding="utf-8")
    await filepath.write_bytes(content)


@router.get("/{github_path:path}", dependencies=[Depends(token_guard)])
async def get_gh(github_path: str = Depends(path_guard)):

    filepath = cache_dir / github_path.replace("/", "_")
    typepath = filepath.with_suffix(filepath.suffix + ".type")
    headers = {}

    if await filepath.exists():
        headers['content-type'] = await typepath.read_text("utf-8")
        return FileResponse(filepath, headers=headers)
    r = await client.get(github_path)
    content = r.content
    headers['content-type'] = r.headers.get("content-type",
                                            "application/octet-stream")
    if r.is_success and len(content) <= settings.file_max:
        await write_file(filepath, content, typepath, headers)
    return Response(content=content, status_code=r.status_code, headers=headers)
