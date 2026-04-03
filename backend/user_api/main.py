import asyncio
from pathlib import Path

from fastapi import FastAPI, Form, WebSocket
import websockets
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from bot.auth import router as auth_router
from bot.app import app as bot_app

app = FastAPI()
app.include_router(auth_router, prefix="/api/auth")

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"]
)

@app.on_event("startup")
async def startup():
    await bot_app.initialize()
    await bot_app.start()
    asyncio.create_task(bot_app.updater.start_polling())

@app.on_event("shutdown")
async def shutdown():
    await bot_app.stop()

# @app.get("/", include_in_schema=False)
# async def root():
#     return { 
#         "status": 200,
#         "version": 1
#     }   

@app.post("/api/user/update")
async def update_exchanges_keys(
    key: str = Form(...)
):
    return {
        "status": 200,
        "message": f"{key} added to user: 1",
    }

@app.get("/api/user/exchanges_keys")
async def get_exchanges_keys():
    return {
        "user": "Vitik1",
        "keys": {
            "api_key": "xxx",
            "api_secret": "xxx",
        } 
    }


@app.websocket("/ws")
async def websocket_proxy(websocket: WebSocket):
    await websocket.accept()
    async with websockets.connect("ws://localhost:9000/ws") as ws_rust:
        async def forward_to_rust():
            async for msg in websocket.iter_text():
                await ws_rust.send(msg)

        async def forward_to_client():
            async for msg in ws_rust:
                await websocket.send_text(msg)

        await asyncio.gather(forward_to_rust(), forward_to_client())

frontend_dist = Path(__file__).parent.parent.parent / "frontend" / "dist"
app.mount("/", StaticFiles(directory=frontend_dist, html=True), name="frontend")