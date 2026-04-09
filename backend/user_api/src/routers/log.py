
import asyncio
from typing import Annotated
from fastapi import APIRouter, Depends, HTTPException
import structlog
import jwt
from jwt.exceptions import InvalidTokenError, InvalidSubjectError

from ..rust_ws import run_ws
from ..schemas import EventDataTypeEnum, EventTypeEnum, AppStatusEnum, MessageContext, MessageData, MessageEventData, MessageEventPayload, MessageMethod, WebSocketActionEnum, WebSocketChannelEnum
from ..cache import push_to_subscribes, user_state
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

    # event_data = {
    #     "type": ,
    #     "tg_user_id": tg_user_id, 
    #     "timestamp": data.timestamp,
    #     "payload": {
    #         "event": data.event, 
    #         "symbol": f"{data.data.symbol.upper()}",
    #         "longExchange": data.data.longExchange,
    #         "longOrderType": data.data.longOrderType,
    #         "shortExchange": data.data.shortExchange,
    #         "shortOrderType": data.data.shortOrderType,
    #         "status": AppStatusEnum.Offline,
    #         "isBotRunning": AppStatusEnum.Stopped
    #     }, 
    # }
    
    match data.event:
        case EventTypeEnum.BotStart:
            if tg_user_id not in user_state or user_state[tg_user_id]["isBotRunning"] == AppStatusEnum.Stopped:
                # Подключаем клиента
                task = asyncio.create_task(run_ws(
                    action=WebSocketActionEnum.Subscribe,
                    channel=WebSocketChannelEnum.OrderBook,
                    long_exchange=data.data.longExchange,
                    short_exchange=data.data.shortExchange,
                    symbol=data.data.symbol,
                    tg_user_id=tg_user_id
                ))
                ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"] = task
                
                # event_data["payload"]["isBotRunning"] = AppStatusEnum.Running
                # event_data["payload"]["status"] = AppStatusEnum.Online

                # Сохраняем насстройки для остальных устройств
                # user_state[tg_user_id] = {
                #     "type": EventDataTypeEnum.UserState,
                #     "symbol": event_data["payload"]["symbol"],
                    
                #     "longExchange": event_data["payload"]["longExchange"],
                #     "longOrderType": event_data["payload"]["longOrderType"],

                #     "shortExchange": event_data["payload"]["shortExchange"],
                #     "shortOrderType": event_data["payload"]["shortOrderType"],

                #     "status": event_data["payload"]["status"],
                #     "isBotRunning": event_data["payload"]["isBotRunning"],
                #     "devices": 1,
                # } 

        case EventTypeEnum.BotStop:
            task = ws_task.get(f"{tg_user_id}:{data.data.symbol.lower()}")
            
            # Отключаем клиента
            if task:
                task.cancel()
                del ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"]
                user_state[tg_user_id]["isBotRunning"] = AppStatusEnum.Stopped
                user_state[tg_user_id]["status"] = AppStatusEnum.Offline

                if user_state[tg_user_id]["devices"] == 0:
                    user_state.pop(tg_user_id, None)
                    log.info(f"LogRouter -> Было удаленно состояние пользователя: {tg_user_id}")
                log.info(f"Для клиента: {tg_user_id}, был отключен RustWebsocket")

    # Пушим новое собитие на все устройства
    await push_to_subscribes(message)

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