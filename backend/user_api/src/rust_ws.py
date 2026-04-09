import asyncio
import json

import websockets
import os
from dotenv import load_dotenv
import structlog

from .cache import push_to_subscribes

from .schemas import (
    EventDataTypeEnum,
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
    symbol: str,
    tg_user_id: int
):
    log.info("Подключение к RustWebsocket")

    try:
        while True:
            try:
                async with websockets.connect(WEBSOCKET_URL) as websocket:
                    log.info("Подключено к RustWebsocket")

                    await websocket.send(json.dumps({
                        "action": action,
                        "channel": channel,
                        "longExchange": long_exchange,
                        "shortExchange": short_exchange,
                        "ticker": symbol
                    }))

                    response = await websocket.recv()
                    data = json.loads(response)
                    data["type"] = EventDataTypeEnum.Websocket
                    await push_to_subscribes(data, tg_user_id=tg_user_id)
                    log.info(data)
            except Exception as e:
                log.error(e)
    except Exception as e:
        log.err(f"RustWebsocket -> {e}")
    except asyncio.CancelledError:
        log.info("RustWebsocket -> успешно остановлен")