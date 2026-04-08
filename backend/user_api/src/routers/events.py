import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

import structlog
from ..cache import event_deque

router = APIRouter()

log: structlog.PrintLogger = structlog.get_logger()

subscribes_users_id = []

async def event_streamer():
    global event_deque

    while True:
        try:
            event = await event_deque.get()
            for user_id in subscribes_users_id:
                if event["tg_user_id"] == user_id or event["tg_user_id"] == "all":
                    log.info(f"User: {event["tg_user_id"]} принял данные")
                    yield f"data: { json.dumps(event) }\n\n"
        except asyncio.TimeoutError as e:
            log.error(f"EventSender: {e}")
            yield f"data: keep-alive\n\n"
 
@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    if tg_user_id not in subscribes_users_id:
        subscribes_users_id.append(tg_user_id)
    
    return StreamingResponse(
        event_streamer(), 
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"}
    )