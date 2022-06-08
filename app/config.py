from pydantic import BaseSettings


class Settings(BaseSettings):
    token: str | None = None
    openapi_url: str = "/openapi.json"

    class Config:
        env_prefix = "simple_gh_"  # defaults to no prefix


settings = Settings()
