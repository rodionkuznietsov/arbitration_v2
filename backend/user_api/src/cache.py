import asyncio

subscribes = {}
async def push_to_subscribes(event_data, tg_user_id: None):
    if tg_user_id is not None:
        for queue in subscribes[tg_user_id]:
            try:
                queue.put_nowait(event_data)
            except asyncio.QueueFull:
                pass  
    else:
        for queues in subscribes.values():
            for queue in queues:
                try:
                    queue.put_nowait(event_data)
                except asyncio.QueueFull:
                    pass