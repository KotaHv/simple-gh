import time
import asyncio
from asyncio import Task

from anyio import Path

from . import util
from ..config import settings
from ..logger import logger

INTERVAL = 10


class BackgroundTask:

    def __init__(self) -> None:
        self.cache_dir = Path(settings.cache_dir)
        self.cache_time = settings.cache_time
        self.max_cache = settings.max_cache
        self.file_size = settings.file_max
        self.task: Task

    async def start(self):
        await self.cache_dir.mkdir(parents=True, exist_ok=True)
        self.task = asyncio.create_task(self.check())

    async def check(self):

        while True:
            await self._check()
            await asyncio.sleep(INTERVAL)

    @logger.catch
    async def _check(self):
        for file in await util.get_dir_files(self.cache_dir,
                                             sort=util.FileSort.CTIME):
            if time.time() - (await file.stat()).st_ctime <= self.cache_time:
                break
            await util.rm_file(file)
            logger.info(f"{file} has been deleted; reason: cache has expired")
        cache_size = await util.get_dir_size(self.cache_dir)
        if cache_size > self.max_cache:
            for file in await util.get_dir_files(self.cache_dir,
                                                 sort=util.FileSort.CTIME):
                cache_size -= await (file.stat()).st_size
                await util.rm_file(file)
                logger.info(
                    f"{file} has been deleted; reason: {cache_size} > {self.max_cache}"
                )
                if cache_size <= self.max_cache:
                    break

    async def stop(self):
        self.task.cancel()
