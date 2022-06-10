import sys
import copy

from loguru import logger as _logger

from .config import settings


__all__ = ["Log", "logger"]

_logger.remove()


def Log(prefix: str) -> _logger:
    settings.log_dir.mkdir(parents=True, exist_ok=True)

    file_path = settings.log_dir / f"{prefix}.log"
    format = f"<green>{{time:YYYY-MM-DD HH:mm:ss.SSS}}</green> | <level>{{level: <8}}</level> | <cyan>{{name}}</cyan>:<cyan>{{function}}</cyan>:<cyan>{{line}}</cyan> | <fg #FFC0CB>{prefix}</fg #FFC0CB> - <level>{{message}}</level>"
    logger = copy.deepcopy(_logger)
    logger.add(sys.stdout, format=format)
    logger.add(file_path, format=format, rotation="00:00", retention="30 days")
    return logger


logger = Log(settings.title)
