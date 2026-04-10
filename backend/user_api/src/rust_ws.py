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
    attempt = 1
    max_attempts = 3
    is_success_running = False

    while attempt <= max_attempts and is_success_running is False:
        try:
            log.info(f"{{ rust_websocket.connect }} -> {attempt} попытка")
            async with websockets.connect(WEBSOCKET_URL) as websocket:
                # Обновляем статус в user_state, для защиты от запусков последующих WebSocket
                is_success_running = True
                
                try:
                    user_state.change_status(
                        tg_user_id=tg_user_id,
                        status=AppStatusEnum.Online,
                        isBotRunning=AppStatusEnum.Running,
                    )

                    log.info(f"{{ rust_websocket.user_state.change_status }} -> {tg_user_id}")
                except AttributeError as e:
                    log.error(f"RustWebsocket {{user_state.change_status)}} -> У {type(e.obj).__name__} нет change_status")
                    log.error(f"RustWebsocket {{user_state.change_status)}} -> Рекомендуем проверить, какие данные передаються в user_state=")
                except Exception as e:
                    log.error(f"RustWebsocket {{user_state.change_status)}} -> {e}")

                await websocket.send(json.dumps({
                    "action": action,
                    "channel": channel,
                    "longExchange": long_exchange,
                    "shortExchange": short_exchange,
                    "ticker": symbol
                }))

            while True:
                log.info("Какие-то данные")
                await asyncio.sleep(1)

                    # response = await websocket.recv()
                    # data = json.loads(response)

    #                 try:
    #                     ws_message = MessageData(
    #                         event_data=MessageEventData(
    #                             type=EventDataTypeEnum.Websocket,
    #                             payload=MessageEventPayload(
    #                                 event=EventTypeEnum.Websocket,
    #                                 symbol=symbol,
    #                                 longExchange=message.event_data.payload.longExchange,
    #                                 longOrderType=message.event_data.payload.longOrderType,
    #                                 shortExchange=message.event_data.payload.shortExchange,
    #                                 shortOrderType=message.event_data.payload.shortOrderType,
    #                                 status=AppStatusEnum.Online,
    #                                 isBotRunning=AppStatusEnum.Running
    #                             ),
    #                             timestamp=int(time()),
    #                             ws_data=data
    #                         ),
    #                         context=MessageContext(
    #                             method=MessageMethod.WebsocketConnected,
    #                             tg_user_id=tg_user_id
    #                         )
    #                     )

    #                     push_to_subscribes(ws_message)
    #                 except Exception as e:
    #                     log.error(f"RustWebsocket -> {e}")                    
        except websockets.exceptions.InvalidStatus as e:            
            # Обновляем статус в user_state, для защиты от запусков последующих WebSocket
            # во время попыток подключения
            try:
                user_state.change_status(
                    tg_user_id=tg_user_id,
                    status=AppStatusEnum.Warning,
                    isBotRunning=AppStatusEnum.Stopped,
                )

                log.info(f"Изменили статус для: {tg_user_id}")
            except AttributeError as e:
                log.error(f"RustWebsocket {{user_state.change_status)}} -> У {type(e.obj).__name__} нет change_status")
                log.error(f"RustWebsocket {{user_state.change_status)}} -> Рекомендуем проверить, какие данные передаються в user_state=")
            except Exception as e:
                log.error(f"RustWebsocket {{user_state.change_status)}} -> {e}")
            
            if e.response.status_code == 502:
                if attempt == max_attempts:
                    log.error(f"{{ rust_websocket.502 }} -> Не удалось подключиться к WebSocket")
                    log.error(f"{{ rust_websocket.502 }} -> Рекомендуем проверить запущен ли WebSocket")

                    # Сбрасываем status UserState, чтобы позже можно было снова попробовать подключиться к WS
                    try:
                        user_state.change_status(
                            tg_user_id=tg_user_id,
                            status=AppStatusEnum.Offline,
                            isBotRunning=AppStatusEnum.Stopped,
                        )

                        log.info(f"{{ rust_websocket.user_state.change_status }} -> {tg_user_id}")
                    except AttributeError as e:
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> У {type(e.obj).__name__} нет change_status")
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> Рекомендуем проверить, какие данные передаються в user_state=")
                    except Exception as e:
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> {e}")

            else:
                log.error(f"{{ rust_websocket.{e.response.status_code} }} -> {e}")
            
            attempt += 1
            await asyncio.sleep(3)
        except Exception as e:
            log.error(f"RustWebsocket -> {e}")
        except asyncio.CancelledError:        
            message = MessageData(
                event_data=MessageEventData(
                    type=EventDataTypeEnum.Websocket,
                    timestamp=int(time()),
                    payload=MessageEventPayload(
                        event=EventTypeEnum.Websocket,
                        symbol=symbol,
                        longExchange=long_exchange,
                        longOrderType=OrderTypeEnum.Spot,
                        shortExchange=short_exchange,
                        shortOrderType=OrderTypeEnum.Spot,
                        isBotRunning=AppStatusEnum.Stopped,
                        status=AppStatusEnum.Offline
                    )
                ),
                context=MessageContext(
                    method=MessageMethod.WebsocketClosed,
                    tg_user_id=tg_user_id,
                )
            )
            push_to_subscribes(message)
            log.info("RustWebsocket -> успешно остановлен")

        