import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

from ..cache import event_deque

router = APIRouter()

async def event_streamer(tg_user_id):
    global event_deque

    while True:
        try:
            event = await event_deque.get()
            if event["tg_user_id"] == tg_user_id or event["tg_user_id"] == "all":
                yield f"data: { json.dumps(event) }\n\n"
        except asyncio.TimeoutError:
            yield f"data: keep-alive\n\n"

@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    return StreamingResponse(event_streamer(tg_user_id), media_type="text/event-stream")