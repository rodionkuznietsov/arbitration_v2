import asyncio
from typing import List
event_deque = asyncio.Queue()
subscribes = List[asyncio.Queue]