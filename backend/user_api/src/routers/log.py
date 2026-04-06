import asyncio
import json

from fastapi import APIRouter, Query
from fastapi.responses import StreamingResponse
import structlog

from ..db_schemas.bot import UserTelegramId

from ..db import database
from ..db_schemas import LogMessageSchema, ResultSchema, UserLogSchema
from ..cache import log_deque

log = structlog.get_logger()
router = APIRouter()

@router.post("/add/log", response_model=ResultSchema, tags=["logs"])
async def add_log(data: UserLogSchema):
    global log_deque

    await database.connect()
    await database.add_log(data)
    await database.close()

    log_event = {
        "tg_user_id": data.data.tg_user_id, 
        "event": data.event, 
        "symbol": data.data.symbol,
        "long_exchange": data.data.long_exchange,
        "short_exchange": data.data.short_exchange,
        "timestamp": data.timestamp
    }
    await log_deque.put(log_event)

    return ResultSchema(
        status_code=200,
        success=True,
        message="Лог был добавлен"
    )

async def log_streamer(tg_user_id: int):
    global log_deque

    while True:
        try:
            log_event = await log_deque.get()
            if log_event["tg_user_id"] == tg_user_id:
                yield f"data: { json.dumps(log_event)}\n\n"
        except asyncio.TimeoutError:
            yield f"data: keep-alive\n\n"

@router.get("/subscribe/logs/{tg_user_id}", response_model=ResultSchema, tags=["logs"])
async def get_new_log(tg_user_id: int):
    return StreamingResponse(log_streamer(tg_user_id), media_type="text/event-stream")

@router.delete("/clear/logs", response_model=ResultSchema, tags=["logs"])
async def clear_all_logs():
    await database.connect()
    await database.clear_table_user_logs()
    await database.close()

    return ResultSchema(
        status_code=200,
        success=True,
        message="Таблица была очищена"
    )

@router.get("/get/logs/{tg_user_id}", response_model=ResultSchema, tags=["logs"])
async def get_logs(tg_user_id: int):
    await database.connect()
    logs = await database.get_user_logs(tg_user_id)
    await database.close()

    if not logs:
        return ResultSchema(
            status_code=404,
            success=False,
            message="История логов не была найдена"
        )

    return ResultSchema(
        status_code=200,
        success=True,
        message=LogMessageSchema(
            logs=logs
        )
    )