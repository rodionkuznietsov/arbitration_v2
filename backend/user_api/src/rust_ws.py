import json

import websockets
import os
from dotenv import load_dotenv
import structlog

load_dotenv()
log: structlog.PrintLogger = structlog.get_logger()

WEBSOCKET_URL = os.getenv("WEBSOCKET_URL")

async def run_ws():
    async with websockets.connect(WEBSOCKET_URL) as websocket:
        await websocket.send(json.dumps({
            "action": "subscribe",
            "channel": "order_book",
            "long_exchange": "Bybit",
            "short_exchange": "Gate.io",
            "symbol": "BTC"
        }))
        response = await websocket.recv()
        log.info(response)