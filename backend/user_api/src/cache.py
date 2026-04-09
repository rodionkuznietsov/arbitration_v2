import asyncio
from collections import defaultdict
from typing import Literal
import structlog

from .schemas import MessageData, MessageMethod

log: structlog.PrintLogger = structlog.get_logger()

subscribes = defaultdict(lambda: {
    "success_queue": [], 
    "error_queue": []
})
user_state = {}

async def push_to_subscribes(
    message: MessageData
):
    try:
        if message.context is not None:
            user_queues = subscribes[message.context.tg_user_id]
            if message.context.method == MessageMethod.User:
                for queues in user_queues["success_queue"]:
                    try:
                        queues.put_nowait(message.event_data)
                    except asyncio.QueueFull:
                        pass
            if message.context.method == MessageMethod.WebsocketErrorConnection:
                for queues in user_queues["error_queue"]:
                    try:
                        queues.put_nowait(message.event_data)
                        log.info("Cache -> Добавлено новое событие для error_queue")
                    except asyncio.QueueFull:
                        pass

    except Exception as e:
        log.error(f"Cache: {e}")

async def get_queue(
    tg_user_id: int
):
    if tg_user_id in subscribes:
        return subscribes[tg_user_id]["error_queue"]
    return None