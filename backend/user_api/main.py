import asyncio
from urllib.request import Request

from fastapi import FastAPI, HTTPException
from fastapi.responses import JSONResponse
from fastapi.encoders import jsonable_encoder

from fastapi.middleware.cors import CORSMiddleware
import structlog

log = structlog.get_logger()

from src.schemas import ResultSchema
from src.routers.tg_bot.auth import database, router as auth_router
from src.routers import log_router
from src import check_active_subscribes, tg_bot_app

from src.routers import exchange_router
from src.routers import user_router, events_router

app = FastAPI(
    title="Arbitrage-Bot Api",
    version="Beta 0.0.5",
    openapi_url="/openapi.json",
    docs_url='/docs'
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://arbitrage-bot.xyz"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"]
)

@app.get("/", include_in_schema=False)
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

app.include_router(user_router, prefix="/user")
app.include_router(auth_router, prefix="/telegram/bot")
app.include_router(log_router, prefix="/user/bot")
app.include_router(events_router)
app.include_router(exchange_router, prefix="/exchanges")

@app.on_event("startup")
async def startup():
    await database.connect(min_size=10, max_size=100)
    
    await tg_bot_app.initialize()
    await tg_bot_app.start()
    asyncio.create_task(tg_bot_app.updater.start_polling())
    # asyncio.create_task(check_active_subscribes())

@app.on_event("shutdown")
async def shutdown():
    await database.close()
    await tg_bot_app.stop()

app.include_router(exchange_router, prefix="/exchanges")
