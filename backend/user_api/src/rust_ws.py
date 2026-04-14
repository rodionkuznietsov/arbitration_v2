import asyncio
import json
from time import time

import websockets
import os
from dotenv import load_dotenv
import structlog
import websockets.connection

from .services.notify_manager import notify_manager

from .schemas.bot import OrderTypeEnum
from .schemas import LogStatusEnum, MessageData, MessageMethod, WebSocketStatuEnum, WebsocketClosedContext
from .cache.cache import push_to_subscribes

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
from .core.state import user_state

load_dotenv()
log: structlog.PrintLogger = structlog.get_logger()

WEBSOCKET_URL = os.getenv("WEBSOCKET_URL")

async def run_ws(
    action: WebSocketActionEnum,
    channel: WebSocketChannelEnum,
    tg_user_id: int,
):
    attempt = 1
    max_attempts = 3
    is_attempt_allow = True
    can_push_message = True

    while attempt <= max_attempts and is_attempt_allow:
        try:
            log.info(f"{{ rust_websocket.connect }} -> {attempt} попытка")
            async with websockets.connect(WEBSOCKET_URL) as websocket:
                # При успешном коннекте завершаем цикл с попытками
                is_attempt_allow = False

                await websocket.send(json.dumps({
                    "action": action,
                    "channel": channel,
                    "longExchange": user_state.long_active_exchange(tg_user_id),
                    "shortExchange": user_state.short_active_exchange(tg_user_id),
                    "ticker": user_state.long_active_symbol(tg_user_id)
                }))

                while True:
                    response = await websocket.recv()
                    data = json.loads(response)
                    notify_manager.push_websocket_message(tg_user_id, data=data)

                    if can_push_message:
                        notify_manager.push_user_state_message(
                            tg_user_id,
                            status=AppStatusEnum.Online
                        )
                        
                        await notify_manager.push_log_message(
                            tg_user_id,
                            event=EventTypeEnum.BotStart,

                            symbol=user_state.long_active_symbol(tg_user_id),

                            longExchange=user_state.long_active_exchange(tg_user_id),
                            longOrderType=user_state.long_active_order_type(tg_user_id),

                            shortExchange=user_state.short_active_exchange(tg_user_id),
                            shortOrderType=user_state.short_active_order_type(tg_user_id),

                            status=LogStatusEnum.Success,
                        )

                        can_push_message = False # Устанавливаем False чтобы сообщение отправилось только один раз

        except websockets.exceptions.InvalidStatus as e:            
            # Обновляем статус в user_state, для защиты от запусков последующих WebSocket
            # во время попыток подключения
            try:
                user_state.change_status(
                    tg_user_id=tg_user_id,
                    status=AppStatusEnum.Warning,
                    isBotRunning=False,
                )

                log.info(f"{{ rust_websocket.user_state.change_status }} -> {tg_user_id}")
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
                            isBotRunning=False,
                        )

                        log.info(f"{{ rust_websocket.user_state.change_status }} -> {tg_user_id}")
                    except AttributeError as e:
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> У {type(e.obj).__name__} нет change_status")
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> Рекомендуем проверить, какие данные передаються в user_state=")
                    except Exception as e:
                        log.error(f"RustWebsocket {{user_state.change_status)}} -> {e}")

            else:
                log.error(f"{{ rust_websocket.{e.response.status_code} }} -> {e}")
            
            notify_manager.push_user_state_message(tg_user_id)

            attempt += 1
            await asyncio.sleep(3)
        except Exception as e:
            # Здесь меняем статус для userState, так мы избежим бага, 
            # запущеного вебсокета после обновление страницы юзером
            try:
                user_state.change_status(
                    tg_user_id=tg_user_id,
                    status=AppStatusEnum.Offline,
                    isBotRunning=False,
                )

                log.info(f"{{ rust_websocket.user_state.change_status }} -> {tg_user_id}")
            except AttributeError as e:
                log.error(f"RustWebsocket {{user_state.change_status)}} -> У {type(e.obj).__name__} нет change_status")
                log.error(f"RustWebsocket {{user_state.change_status)}} -> Рекомендуем проверить, какие данные передаються в user_state=")
            except Exception as e:
                log.error(f"RustWebsocket {{user_state.change_status)}} -> {e}")

            notify_manager.push_user_state_message(tg_user_id)
            log.error(f"{{ rust_websocket.error }} -> принудительно остановлен: {e}")

            attempt += 1    
            await asyncio.sleep(3)
        except asyncio.CancelledError: # Юзер отключил WS
            user_state.change_status(
                tg_user_id=tg_user_id,
                status=AppStatusEnum.Offline,
                isBotRunning=False,
            )
            
            notify_manager.push_user_state_message(tg_user_id)

            log.info("RustWebsocket -> успешно остановлен")

        