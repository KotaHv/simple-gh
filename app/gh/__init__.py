import httpx
from fastapi import APIRouter, HTTPException, BackgroundTasks, Depends
from fastapi.responses import FileResponse, StreamingResponse
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


@router.get("/{github_path:path}", dependencies=[Depends(token_guard)])
async def get_gh(github_path: str = Depends(path_guard)):

    filepath = cache_dir / github_path.replace("/", "_")
    typepath = filepath.with_suffix(filepath.suffix + ".type")
    headers = {}

    if await filepath.exists():
        headers['content-type'] = await typepath.read_text("utf-8")
        return FileResponse(filepath, headers=headers)

    req = client.build_request("GET", github_path)
    r = await client.send(req, stream=True)
    headers['content-type'] = r.headers.get("content-type",
                                            "application/octet-stream")

    async def write_file():
        await typepath.write_text(headers["content-type"], encoding="utf-8")
        async with await filepath.open("wb") as f:
            async for chunk in r.aiter_bytes():
                await f.write(chunk)
                yield chunk

    stream_fn = r.aiter_bytes
    if r.is_success:
        stream_fn = write_file
    return StreamingResponse(stream_fn(),
                             headers=headers,
                             status_code=r.status_code,
                             background=BackgroundTasks([r.aclose]))
