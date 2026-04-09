import asyncio
from collections import defaultdict
from typing import Literal
import structlog

from .schemas import MessageData

log: structlog.PrintLogger = structlog.get_logger()

subscribes = defaultdict(lambda: {"success_queue": [], "error_queue": []})
user_state = {}

async def push_to_subscribes(
    message: MessageData
):
    try:
            log.info(subscribes)

        # for queues in subscribes:
            # for queue in queues:
            #     try:
            #         if message.context is not None:
                        
            #         # queue.put_nowait(event_data)
            #     except asyncio.QueueFull:
            #         pass
    except Exception as e:
        log.error(f"Cache: {e}")