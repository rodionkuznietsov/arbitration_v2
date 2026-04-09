import asyncio
import json

import websockets
import os
from dotenv import load_dotenv
import structlog

from .schemas import (
    WebSocketActionEnum, 
    WebSocketChannelEnum, 
    ExchangeEnum
)

load_dotenv()
log: structlog.PrintLogger = structlog.get_logger()

WEBSOCKET_URL = os.getenv("WEBSOCKET_URL")

async def run_ws(
    action: WebSocketActionEnum,
    channel: WebSocketChannelEnum,
    long_exchange: ExchangeEnum,
    short_exchange: ExchangeEnum,
    symbol: str
):
    try:
        async with websockets.connect(WEBSOCKET_URL) as websocket:
            await websocket.send(json.dumps({
                "action": action,
                "channel": channel,
                "longExchange": long_exchange,
                "shortExchange": short_exchange,
                "ticker": symbol
            }))

            while True:
                response = await websocket.recv()
                log.info(response)
    except asyncio.asyncio.CancelledError:
        log.info("ВебСокет остановлен")