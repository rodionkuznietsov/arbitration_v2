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

app = FastAPI(
    title="Arbitrage-Bot Api",
    version="Beta 0.0.5",
    openapi_url="/api/openapi.json",
    docs_url='/api/docs'
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://arbitrage-bot.xyz"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"]
)

@app.get("/api")
async def start_page():
    return ResultSchema(
        status_code=200,
        success=True,
        message="Api is working"
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
app.include_router(events_router, prefix='/api')
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
