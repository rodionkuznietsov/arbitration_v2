
import asyncio
from time import time
from typing import Annotated
from fastapi import APIRouter, Depends, HTTPException
import structlog
import jwt
from jwt.exceptions import InvalidTokenError, InvalidSubjectError

from ..rust_ws import run_ws
from ..schemas import EventDataTypeEnum, EventTypeEnum, AppStatusEnum, MessageContext, MessageData, MessageEventData, MessageEventPayload, MessageMethod, UserStatePayload, WebSocketActionEnum, WebSocketChannelEnum
from ..cache import get_queue, push_to_subscribes, user_state
from ..jwt_func import ALGORITHM, JWT_SECRET_KEY, oauth2_scheme
from ..db import database
from ..schemas import LogMessageSchema, ResultSchema, UserLogSchema

log: structlog.PrintLogger = structlog.get_logger()
router = APIRouter()

ws_task = {}

@router.post("/add/log", response_model=ResultSchema, tags=["logs"])
async def add_log(data: UserLogSchema, token: Annotated[str, Depends(oauth2_scheme)], ):
    global log_deque

    tg_user_id = int(authothicate(token))
    await database.add_log(tg_user_id, data)

    message = MessageData(
        event_data=MessageEventData(
            type=EventDataTypeEnum.Log,
            timestamp=data.timestamp,
            payload=MessageEventPayload(
                event=data.event,
                symbol=f"{data.data.symbol.upper()}",
                longExchange=data.data.longExchange,
                longOrderType=data.data.longOrderType,
                shortExchange=data.data.shortExchange,
                shortOrderType=data.data.shortOrderType,
                isBotRunning=AppStatusEnum.Stopped,
                status=AppStatusEnum.Offline
            )
        ),
        context=MessageContext(
            method=MessageMethod.User,
            tg_user_id=tg_user_id,
        )
    )

    match data.event:
        case EventTypeEnum.BotStart:
            if tg_user_id not in user_state or user_state[tg_user_id].event_data.payload.isBotRunning == AppStatusEnum.Stopped:
                # Сохраняем насстройки для остальных устройств
                user_state[tg_user_id] = MessageData(
                    event_data=MessageEventData(
                        type=EventTypeEnum.UserState,
                        payload=UserStatePayload(
                            symbol=message.event_data.payload.symbol,
                            
                            longExchange=message.event_data.payload.longExchange,
                            longOrderType=message.event_data.payload.longOrderType,

                            shortExchange=message.event_data.payload.shortExchange,
                            shortOrderType=message.event_data.payload.shortOrderType,

                            status=message.event_data.payload.status,
                            isBotRunning=message.event_data.payload.isBotRunning,
                        ),
                        timestamp=int(time())
                    ),
                    context=MessageContext(
                        method=MessageMethod.User,
                        tg_user_id=tg_user_id
                    )
                )
                
                # Подключаем клиента
                task = asyncio.create_task(run_ws(
                    action=WebSocketActionEnum.Subscribe,
                    channel=WebSocketChannelEnum.OrderBook,
                    long_exchange=data.data.longExchange,
                    short_exchange=data.data.shortExchange,
                    symbol=data.data.symbol,
                    user_state=user_state[tg_user_id],
                    message=message
                ))
                ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"] = task

                log.info(user_state[tg_user_id].event_data.payload.status)

                # error_queues = await get_queue(tg_user_id)
                # for queue in error_queues:
                #     error_event = await queue.get_nowait()
                #     message.event_data.payload.isBotRunning = error_event.payload.isBotRunning
                #     message.event_data.payload.status = error_event.payload.status

                #     user_state[tg_user_id].event_data.payload.isBotRunning = error_event.payload.isBotRunning
                #     user_state[tg_user_id].event_data.payload.status = error_event.payload.status

        case EventTypeEnum.BotStop:
            task = ws_task.get(f"{tg_user_id}:{data.data.symbol.lower()}")
            
            # Отключаем клиента
            if task:
                task.cancel()
                del ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"]
                if tg_user_id in user_state:
                    user_state[tg_user_id].event_data.payload.isBotRunning = AppStatusEnum.Stopped
                    user_state[tg_user_id].event_data.payload.status = AppStatusEnum.Offline

                log.info(f"Для клиента: {tg_user_id}, был отключен RustWebsocket")

    # Пушим новое собитие на все устройства
    # push_to_subscribes(message)

    return ResultSchema(
        status_code=200,
        success=True,
        message="Лог был добавлен"
    )

@router.delete("/clear/logs", response_model=ResultSchema, tags=["logs"])
async def clear_all_logs():
    await database.clear_table_user_logs()

    return ResultSchema(
        status_code=200,
        success=True,
        message="Таблица была очищена"
    )

@router.get("/get/logs/", response_model=ResultSchema, tags=["logs"])
async def get_logs(token: Annotated[str, Depends(oauth2_scheme)]):
    tg_user_id = int(authothicate(token))
    logs = await database.get_user_logs(tg_user_id)

    if not logs:
        raise HTTPException(
            status_code=404,
            detail="Не удалось найти логов"
        )

    return ResultSchema(
        status_code=200,
        success=True,
        message=LogMessageSchema(
            logs=logs
        )
    )

def authothicate(token):
    credentials_exception = HTTPException(
        status_code=401,
        detail="Не удалось проверить учетные данные",
        headers={"WWW-Authenticate": "Bearer"}
    )

    try:
        payload = jwt.decode(token, JWT_SECRET_KEY, algorithms=[ALGORITHM])
    except InvalidSubjectError as e:
        log.error(f"JWT TYPE ERROR: {e}")
        raise credentials_exception
    except InvalidTokenError as e:
        log.error(f"JWT TYPE ERROR: {e}")
        raise credentials_exception

    tg_user_id = payload.get('sub')
    if tg_user_id is None:
        raise credentials_exception
    
    return tg_user_id