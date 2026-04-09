import asyncio
from typing import Literal
import structlog

from .schemas import MessageData

log: structlog.PrintLogger = structlog.get_logger()

subscribes = {}
user_state = {}

async def push_to_subscribes(
    message: MessageData
):
    try:
        pass
        # if error_message is not None:
        #     for queue in subscribes[tg_user_id]:
        #         try:
        #             queue.put_nowait(event_data)
        #         except asyncio.QueueFull:
        #             pass  
        # else:
            # for queues in subscribes.values():
            #     for queue in queues:
            #         try:
            #             queue.put_nowait(event_data)
            #         except asyncio.QueueFull:
            #             pass
    except Exception as e:
        log.error(f"Cache: {e}")