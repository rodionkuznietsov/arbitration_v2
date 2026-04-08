import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

import structlog
from ..cache import subscribes

router = APIRouter()

log: structlog.PrintLogger = structlog.get_logger()

async def event_streamer(data: asyncio.Queue):
    while True:
        try:
            event = await data.get()
            yield f"data: { json.dumps(event) }\n\n"
        except asyncio.TimeoutError as e:
            log.error(f"EventSender: {e}")
            yield f"data: keep-alive\n\n"
 
@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    q = asyncio.Queue()
    subscribes[tg_user_id] = q
    
    return StreamingResponse(
        event_streamer(q), 
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"}
    )

async def push_to_subscribes(event_data):
    print(len(subscribes))
    for q in subscribes.values():
        await q.put(event_data)