import re

B = 1
KB = 1024
MB = KB * 1024
GB = MB * 1024
TB = GB * 1024
size_dict = {
    "B": B,
    "KB": KB,
    "MB": MB,
    "GB": GB,
    "TB": TB,
}


def convert_bytes(data: str | int) -> int | float:
    if isinstance(data, (int, float)):
        return data
    regx = re.compile(r"(\d*\.)?\d+")
    num_str = regx.match(data).group()
    num = float(num_str) if num_str.find(".") != -1 else int(num_str)
    unit = regx.sub("", data).upper()
    return num * size_dict[unit]
