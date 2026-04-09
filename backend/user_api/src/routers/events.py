import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

import structlog
from ..cache import push_to_subscribes, subscribes, user_state

router = APIRouter()

log: structlog.PrintLogger = structlog.get_logger()

async def event_streamer(data: asyncio.Queue, tg_user_id):
    try:
        while True:
            event = await data.get()
            yield f"data: { json.dumps(event) }\n\n"
    except asyncio.CancelledError:
        pass
    finally:
        # убираем только свою очередь
        if tg_user_id in subscribes and data in subscribes[tg_user_id]:
            subscribes[tg_user_id].remove(data)
            log.info(
                f"EventsRouter -> {tg_user_id} вышел из потока. "
                f"Его текущие потоки: {len(subscribes.get(tg_user_id, []))}"
            )

@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    try:        
        if tg_user_id not in subscribes:
            subscribes[tg_user_id]["success_queue"] = []
            subscribes[tg_user_id]["error_queue"] = []

        success_queue = asyncio.Queue()
        error_queue = asyncio.Queue()
        subscribes[tg_user_id]["success_queue"].append(success_queue)
        subscribes[tg_user_id]["error_queue"].append(error_queue)
        
        if tg_user_id in user_state:
            user_state[tg_user_id]["devices"] = user_state[tg_user_id]["devices"] + 1
            log.info(f"UserState: {user_state}")

            await push_to_subscribes(user_state[tg_user_id], tg_user_id=tg_user_id)

        return StreamingResponse(
            event_streamer(q, tg_user_id), 
            media_type="text/event-stream",
            headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"}
        )
    except Exception as e:
        log.error(f"EventsRouter -> {e}")