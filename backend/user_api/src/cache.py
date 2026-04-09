import asyncio
from collections import defaultdict
from typing import Literal
import structlog

from .schemas import MessageData, MessageMethod

log: structlog.PrintLogger = structlog.get_logger()

subscribes = defaultdict(lambda: {"success_queue": [], "error_queue": []})
user_state = {}

async def push_to_subscribes(
    message: MessageData
):
    try:
        if message.context is not None:
            for queues in subscribes[message.context.tg_user_id]:
                try:
                    if message.context.method == MessageMethod.User:
                        queues["success_queue"].put_nowait(message.event_data)
                except asyncio.QueueFull:
                    pass

        # for queues in subscribes:
            # for queue in queues:
            #     try:
            #         if message.context is not None:
                        
            #         # queue.put_nowait(event_data)
            #     except asyncio.QueueFull:
            #         pass
    except Exception as e:
        log.error(f"Cache: {e}")