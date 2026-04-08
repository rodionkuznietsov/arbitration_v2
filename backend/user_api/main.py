import asyncio
from pathlib import Path
from urllib.request import Request

from fastapi import FastAPI, WebSocket, HTTPException
from fastapi.responses import JSONResponse
from fastapi.encoders import jsonable_encoder

import websockets
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles
import structlog

log = structlog.get_logger()

from src.schemas import ResultSchema
from src.routers.tg_bot.auth import database, router as auth_router
from src.routers import log_router
from src import tg_bot_app

from src.routers import exchange_router
from src.routers import user_router, events_router

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://arbitrage-bot.xyz"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"]
)

@app.exception_handler(HTTPException)
async def http_exception_handler(request: Request, exc: HTTPException):
    return JSONResponse(
        status_code=exc.status_code,
        content=jsonable_encoder(ResultSchema(
            status_code=exc.status_code,
            success=False,
            message=exc.detail,
        ))
    )

app.include_router(user_router, prefix="/api/user")
app.include_router(auth_router, prefix="/api/telegram/bot")
app.include_router(log_router, prefix="/api/user/bot")
app.include_router(events_router, prefix="/api")
app.include_router(exchange_router, prefix="/api/exchanges")

@app.on_event("startup")
async def startup():
    await database.connect(min_size=10, max_size=100)
    
    await tg_bot_app.initialize()
    await tg_bot_app.start()
    asyncio.create_task(tg_bot_app.updater.start_polling())

@app.on_event("shutdown")
async def shutdown():
    await database.close()
    await tg_bot_app.stop()

app.include_router(exchange_router, prefix="/api/exchanges")

@app.websocket("/ws")
async def websocket_proxy(websocket: WebSocket):
    await websocket.accept()
    async with websockets.connect("ws://localhost:9000/ws") as ws_rust:
        async def forward_to_rust():
            async for msg in websocket.iter_text():
                await ws_rust.send(msg)
    
        async def forward_to_client():
            async for msg in ws_rust:
                try:
                    if websocket.client_state.value == 1:  # WebSocketState.CONNECTED
                        await websocket.send_text(msg)
                    else:
                        await ws_rust.close()
                except RuntimeError as _:
                    log.error("WebsocketProxy -> Нельзя отправить сообщение, соединение закрыто")
                        
        await asyncio.gather(forward_to_rust(), forward_to_client())

frontend_dist = Path(__file__).parent.parent.parent / "frontend" / "dist"
app.mount("/", StaticFiles(directory=frontend_dist, html=True), name="frontend")
