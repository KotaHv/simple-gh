from pydantic import BaseSettings, ByteSize, DirectoryPath

DEFAULT_DATA_DIR = "data"
DEFAULT_LOG_DIR = DEFAULT_DATA_DIR + "/logs"
DEFAULT_CACHE_DIR = DEFAULT_DATA_DIR + "/cache"


class Settings(BaseSettings):
    token: str | None = None
    openapi_url: str = ""
    title: str = "simple_gh"

    max_cache: ByteSize = "512MiB"
    file_max: ByteSize = 20 * 1024 * 1024
    cache_time: int = 60 * 60 * 24

    data_dir: DirectoryPath = DEFAULT_DATA_DIR
    log_dir: DirectoryPath = DEFAULT_LOG_DIR
    cache_dir: DirectoryPath = DEFAULT_CACHE_DIR

    class Config:
        env_prefix = "simple_gh_"  # defaults to no prefix


settings = Settings()
