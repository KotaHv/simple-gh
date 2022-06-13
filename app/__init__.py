from fastapi import FastAPI, Request
from fastapi.exception_handlers import http_exception_handler
from starlette.exceptions import HTTPException as StarletteHTTPException


from .config import settings
from . import gh
from .logger import logger


def create_app():
    app = FastAPI(openapi_url=settings.openapi_url)
    app.add_event_handler("shutdown", close_httpx_client)
    app.include_router(gh.router, prefix="/gh")
    register_exception(app)

    @app.get("/healthcheck")
    async def healthcheck():
        return {"healthcheck": "ok"}

    return app


async def close_httpx_client():
    await gh.client.aclose()


def register_exception(app: FastAPI):
    @app.exception_handler(StarletteHTTPException)
    async def custom_http_exception_handler(
        request: Request, exc: StarletteHTTPException
    ):
        logger.error(
            f"IP Address: {request.headers.get('X-Forwarded-For',request.client.host)} - code: {exc.status_code} - detail: {exc.detail} -  request url: {request.url} - headers:{request.headers}"
        )
        return await http_exception_handler(request, exc)
