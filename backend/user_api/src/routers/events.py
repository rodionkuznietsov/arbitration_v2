import asyncio
import json

from fastapi import APIRouter
from fastapi.responses import StreamingResponse

import structlog

from ..services.notify_manager import notify_manager

from ..schemas import AppStatusEnum
from ..cache import subscribes
from ..core.state import user_state

router = APIRouter()

log: structlog.PrintLogger = structlog.get_logger()

async def event_streamer(data: asyncio.Queue, tg_user_id):
    try:
        while True:
            event = await data.get()
            yield f"data: { json.dumps(event.dict()) }\n\n"
            # log.info(f"Отправили событие { event }")
    except asyncio.CancelledError:
        pass
    except Exception as e:
        log.error(f"EventsRouter -> {e}")

    finally:
        # Обновляем статус isSleeping, чтобы защитить от удаления нужных очередей, пока бот активен 
        user_state.change_sleeping_status(tg_user_id=tg_user_id, new_status=AppStatusEnum.Sleeping)
        log.info(f"{{ events_router.user_state.change_sleeping_status }} -> {tg_user_id}")

@router.get("/subscribe/events/{tg_user_id}", tags=["events"])
async def subscribe_events(tg_user_id: int):
    try:        
        success_queue = asyncio.Queue(maxsize=1000)
        error_queue = asyncio.Queue(maxsize=1000)
        
        subscribes[tg_user_id]["success_queue"].append(success_queue)
        subscribes[tg_user_id]["error_queue"].append(error_queue)
        
        # Возращаем состояние юзера
        if tg_user_id in user_state.exists_users():
            log.info(f"{{ events_router.subscribe.events.init_data }} -> {tg_user_id}")
            
            # Обновляем статус на NotSleeping, чтобы в cache.check_active_subscribes не удалялись очереди, если юзер активный
            user_state.change_sleeping_status(tg_user_id, new_status=AppStatusEnum.NotSleeping)
            log.info(user_state.get(tg_user_id))

            notify_manager.push_user_state_message(tg_user_id)

        return StreamingResponse(
            event_streamer(success_queue, tg_user_id), 
            media_type="text/event-stream",
            headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"}
        )
    except Exception as e:
        log.error(f"EventsRouter -> {e}")