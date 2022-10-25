from fastapi import Request


def get_ip(request: Request) -> str:
    if ip := request.headers.get("X-Forwarded-For"):
        ip = ip.split(",", maxsplit=1)
        return ip[0]
    if ip := request.headers.get("X-Real-IP"):
        return ip
    if ip := request.client:
        return f"{ip.host}:{ip.port}"
    return "Unknown"
