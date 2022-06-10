from pydantic import BaseSettings

DEFAULT_DATA_DIR = "data"
DEFAULT_LOG_DIR = DEFAULT_DATA_DIR + "/logs"
DEFAULT_CACHE_DIR = DEFAULT_DATA_DIR + "/cache"


class Settings(BaseSettings):
    token: str | None = None
    openapi_url: str = "/openapi.json"
    title: str = "simple_gh"

    max_cache: int = 1024 * 1024 * 512
    data_dir: str = DEFAULT_DATA_DIR
    log_dir: str = DEFAULT_LOG_DIR
    cache_dir: str = DEFAULT_CACHE_DIR
    

    class Config:
        env_prefix = "simple_gh_"  # defaults to no prefix


settings = Settings()
