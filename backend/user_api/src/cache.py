import asyncio
from collections import defaultdict
import structlog

from .services import UserState

from .schemas import AppStatusEnum, MessageData, MessageMethod

log: structlog.PrintLogger = structlog.get_logger()

subscribes = defaultdict(lambda: {
    "success_queue": [], 
    "error_queue": []
})
user_state = {}
user_state1 = UserState()

def push_to_subscribes(
    message: MessageData
):
    try:
        if message.context is not None:
            user_queues = subscribes[message.context.tg_user_id]
            if message.context.method == MessageMethod.WebsocketErrorConnection:
                for queues in user_queues["error_queue"]:
                    try:
                        queues.put_nowait(message.event_data)
                    except asyncio.QueueFull:
                        pass
            else:
                for queues in user_queues["success_queue"]:
                    try:
                        queues.put_nowait(message.event_data)
                    except asyncio.QueueFull:
                        pass

    except Exception as e:
        log.error(f"Cache: {e}")

async def get_queue(
    tg_user_id: int
):
    if tg_user_id in subscribes:
        return subscribes[tg_user_id]["success_queue"]
    return None

async def check_active_subscribes():
    while True:
        try:
            for user in user_state.values():
                # Удаляем все очереди для юзера
                if user.event_data.payload.isBotRunning != AppStatusEnum.Running:
                    subscribes[user.context.tg_user_id]["success_queue"].clear()
                    subscribes[user.context.tg_user_id]["error_queue"].clear()
                    
                    log.info(f"Очищены все очереди для: {user.context.tg_user_id}")

        except Exception as e:
            log.error(f"Cache -> {e}")
        await asyncio.sleep(20)