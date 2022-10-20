import time
import http
from datetime import datetime

from fastapi import FastAPI, Request, Response
from fastapi.exception_handlers import http_exception_handler
from starlette.exceptions import HTTPException as StarletteHTTPException

from .config import settings
from . import gh
from .gh import background
from .logger import logger

LOGGING_ROUTE_BLACKLIST = ["/alive"]


def create_app():
    app = FastAPI(openapi_url=settings.openapi_url)
    app.add_event_handler("shutdown", close_httpx_client)
    app.include_router(gh.router, prefix="/gh")
    register_middleware(app)
    register_exception(app)
    register_event(app)

    @app.get("/alive")
    async def alive():
        return {"msg": datetime.now()}

    return app


def register_event(app: FastAPI):
    bt = background.BackgroundTask()

    @app.on_event("startup")
    async def startup_event():
        await bt.start()
        return

    @app.on_event("shutdown")
    async def shutdown_event():
        await bt.stop()
        return


async def close_httpx_client():
    await gh.client.aclose()


def register_exception(app: FastAPI):

    @app.exception_handler(StarletteHTTPException)
    async def custom_http_exception_handler(request: Request,
                                            exc: StarletteHTTPException):

        logger.error(f"[{exc.status_code}] [{exc.detail}] [{exc.headers}]")
        return await http_exception_handler(request, exc)


def register_middleware(app: FastAPI):

    @app.middleware("http")
    async def http_middleware(request: Request, call_next):
        start_time = time.perf_counter_ns()
        response: Response = await call_next(request)
        path = request.url.path
        for route in LOGGING_ROUTE_BLACKLIST:
            if path.startswith(route):
                return response
        process_time = time.perf_counter_ns() - start_time  # ns
        process_time = process_time / 1000 / 1000  # ms
        process_time = f"{process_time:.4f} ms"
        response.headers["X-Process-Time"] = process_time
        client = f"{request.client.host}:{request.client.port}"
        ip = request.headers.get('X-Real-IP', client)
        status = f"{response.status_code} {http.HTTPStatus(response.status_code).phrase}"
        if response.status_code >= 400:
            ua = request.headers.get("user-agent", "Unknown")
            logger.error(
                f"{ip} {request.method} {path} => {status} [{ua}] {process_time}"
            )
        logger.info(f"{ip} {request.method} {path} => {status} {process_time}")
        return response
