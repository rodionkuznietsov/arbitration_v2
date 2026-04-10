import asyncio
import json
from time import time

import pydantic
import websockets
import os
from dotenv import load_dotenv
import structlog
import traceback

from .services import UserState

from .schemas.bot import OrderTypeEnum

from .cache import MessageData, MessageMethod, push_to_subscribes

from .schemas import (
    AppStatusEnum,
    EventDataTypeEnum,
    EventTypeEnum,
    MessageContext,
    MessageEventData,
    MessageEventPayload,
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
    
    user_state: UserState,
    tg_user_id: int,
    message: MessageData,
):
    log.info("Подключение к RustWebsocket")

    try:
        user_state.change_status(
            tg_user_id=tg_user_id,
            status=AppStatusEnum.Online,
            isBotRunning=AppStatusEnum.Running,
        )

        log.info("Status changed")
    except Exception as e:
        log.error(f"RustWebsocket(UserState) -> {e.args}")

    # try:
    #     while True:
    #         try:
    #             async with websockets.connect(WEBSOCKET_URL) as websocket:
    #                 # Обновляем статус в user_state, для защиты от запусков последующих WebSocket
    #                 user_state.change_status(
    #                     tg_user_id=tg_user_id,
    #                     status=AppStatusEnum.Online,
    #                     isBotRunning=AppStatusEnum.Running,
    #                 )

    #                 log.info(user_state.get(tg_user_id))

    #                 # await websocket.send(json.dumps({
    #                 #     "action": action,
    #                 #     "channel": channel,
    #                 #     "longExchange": long_exchange,
    #                 #     "shortExchange": short_exchange,
    #                 #     "ticker": symbol
    #                 # }))

    #                 # response = await websocket.recv()
    #                 # data = json.loads(response)

    #                 # try:
    #                 #     ws_message = MessageData(
    #                 #         event_data=MessageEventData(
    #                 #             type=EventDataTypeEnum.Websocket,
    #                 #             payload=MessageEventPayload(
    #                 #                 event=EventTypeEnum.Websocket,
    #                 #                 symbol=symbol,
    #                 #                 longExchange=message.event_data.payload.longExchange,
    #                 #                 longOrderType=message.event_data.payload.longOrderType,
    #                 #                 shortExchange=message.event_data.payload.shortExchange,
    #                 #                 shortOrderType=message.event_data.payload.shortOrderType,
    #                 #                 status=AppStatusEnum.Online,
    #                 #                 isBotRunning=AppStatusEnum.Running
    #                 #             ),
    #                 #             timestamp=int(time()),
    #                 #             ws_data=data
    #                 #         ),
    #                 #         context=MessageContext(
    #                 #             method=MessageMethod.WebsocketConnected,
    #                 #             tg_user_id=tg_user_id
    #                 #         )
    #                 #     )

    #                 #     push_to_subscribes(ws_message)
    #                 # except Exception as e:
    #                 #     log.error(f"RustWebsocket -> {e}")                    
    #         except websockets.exceptions.InvalidStatus as e:
    #             log.error(f"RustWebsocket -> {e}")

    #             try:
    #                 message = MessageData(
    #                     event_data=MessageEventData(
    #                         type=EventDataTypeEnum.Log,
    #                         timestamp=int(time()),
    #                         payload=MessageEventPayload(
    #                             event=EventTypeEnum.BotStart,
    #                             symbol=symbol,
    #                             longExchange=long_exchange,
    #                             longOrderType=OrderTypeEnum.Spot,
    #                             shortExchange=short_exchange,
    #                             shortOrderType=OrderTypeEnum.Spot,
    #                             isBotRunning=AppStatusEnum.Stopped,
    #                             status=AppStatusEnum.Warning
    #                         )
    #                     ),
    #                     context=MessageContext(
    #                         method=MessageMethod.WebsocketErrorConnection,
    #                         tg_user_id=tg_user_id,
    #                     )
    #                 )

    #                 push_to_subscribes(message=message)
    #             except pydantic.ValidationError as e:
    #                 log.error(f"RustWebsocket -> {e}")

    #             break
    # except Exception as e:
    #     log.err(f"RustWebsocket -> {e}")
    # except asyncio.CancelledError:        
    #     message = MessageData(
    #         event_data=MessageEventData(
    #             type=EventDataTypeEnum.Websocket,
    #             timestamp=int(time()),
    #             payload=MessageEventPayload(
    #                 event=EventTypeEnum.Websocket,
    #                 symbol=symbol,
    #                 longExchange=long_exchange,
    #                 longOrderType=OrderTypeEnum.Spot,
    #                 shortExchange=short_exchange,
    #                 shortOrderType=OrderTypeEnum.Spot,
    #                 isBotRunning=AppStatusEnum.Stopped,
    #                 status=AppStatusEnum.Offline
    #             )
    #         ),
    #         context=MessageContext(
    #             method=MessageMethod.WebsocketClosed,
    #             tg_user_id=tg_user_id,
    #         )
    #     )
    #     push_to_subscribes(message)
    #     log.info("RustWebsocket -> успешно остановлен")