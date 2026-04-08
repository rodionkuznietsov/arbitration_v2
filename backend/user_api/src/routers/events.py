import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

import structlog

from ..cache import event_deque

router = APIRouter()

log: structlog.PrintLogger = structlog.get_logger()

async def event_streamer(tg_user_id: int):
    global event_deque

    while True:
        try:
            event = await event_deque.get()
            if event["tg_user_id"] == tg_user_id or event["tg_user_id"] == "all":
                log.info(f"User: {event["tg_user_id"]} принял данные")
                yield f"data: { json.dumps(event) }\n\n"
        except asyncio.TimeoutError as e:
            log.error(f"EventSender: {e}")
            yield f"data: keep-alive\n\n"
 
@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    return StreamingResponse(
        event_streamer(tg_user_id), 
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"}
    )