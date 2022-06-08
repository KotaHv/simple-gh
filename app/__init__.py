import httpx
from fastapi import FastAPI

client = httpx.AsyncClient()


def create_app():
    app = FastAPI()
    app.add_event_handler("shutdown", close_httpx_client)
    mount(app)
    return app


async def close_httpx_client():
    await client.aclose()


def mount(app: FastAPI):
    from . import gh

    app.mount("/gh", gh.router)
