from pydantic import BaseSettings

from .utils import convert_bytes

DEFAULT_DATA_DIR = "data"
DEFAULT_LOG_DIR = DEFAULT_DATA_DIR + "/logs"
DEFAULT_CACHE_DIR = DEFAULT_DATA_DIR + "/cache"


class Settings(BaseSettings):
    token: str | None = None
    openapi_url: str = "/openapi.json"
    title: str = "simple_gh"

    max_cache: int | float | str = 512 * 1024 * 1024
    file_max: int | float | str = 20 * 1024 * 1024
    cache_time: int = 60 * 60 * 24

    data_dir: str = DEFAULT_DATA_DIR
    log_dir: str = DEFAULT_LOG_DIR
    cache_dir: str = DEFAULT_CACHE_DIR

    class Config:
        env_prefix = "simple_gh_"  # defaults to no prefix


settings = Settings()
settings.max_cache = convert_bytes(settings.max_cache)
settings.file_max = convert_bytes(settings.file_max)
