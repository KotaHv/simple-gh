from fastapi import FastAPI

from .config import settings
from . import gh


def create_app():
    app = FastAPI(openapi_url=settings.openapi_url)
    app.add_event_handler("shutdown", close_httpx_client)
    mount(app)

    @app.get("/healthcheck")
    async def healthcheck():
        return {"healthcheck": "ok"}

    return app


async def close_httpx_client():
    await gh.client.aclose()


def mount(app: FastAPI):

    app.include_router(gh.router, prefix="/gh")
