from app import create_app

_app = create_app()

if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        app="main:_app",
        host="0.0.0.0",
        port=5000,
        reload=True,
        debug=True,
        # ssl_keyfile='192.168.1.232+3-key.pem',
        # ssl_certfile='192.168.1.232+3.pem'
    )
