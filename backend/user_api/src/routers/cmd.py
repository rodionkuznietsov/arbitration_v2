
import asyncio
from time import time
from typing import Annotated
from fastapi import APIRouter, Depends, HTTPException
import structlog
import jwt
from jwt.exceptions import InvalidTokenError, InvalidSubjectError


from ..rust_ws import run_ws
from ..schemas import EventDataTypeEnum, EventTypeEnum, AppStatusEnum, LogPayload, LogStatusEnum, MessageContext, MessageData, MessageEventData, MessageEventPayload, MessageMethod, UserStatePayload, WebSocketActionEnum, WebSocketChannelEnum
from ..cache import push_to_subscribes
from ..core.state import user_state
from ..jwt_func import oauth2_scheme
from ..db import database
from ..schemas import LogMessageSchema, ResultSchema, UserLogSchema
from ..services import authothicate

log: structlog.PrintLogger = structlog.get_logger()
router = APIRouter()

ws_task = {}

@router.post("/add/log", response_model=ResultSchema, tags=["logs"])
async def add_log(data: UserLogSchema, token: Annotated[str, Depends(oauth2_scheme)]):
    global log_deque

    tg_user_id = int(authothicate(token))
    await database.add_log(tg_user_id, data)

    match data.event:
        case EventTypeEnum.BotStart:
            if not user_state.isBotRunning(tg_user_id) and user_state.status(tg_user_id) != AppStatusEnum.Warning:
                log.info("Меняем данные")
                user_state.set_active_from_draft(tg_user_id)
                user_state.get_active_config(tg_user_id)

                # Подключаем клиента
                task = asyncio.create_task(run_ws(
                    action=WebSocketActionEnum.Subscribe,
                    channel=WebSocketChannelEnum.OrderBook,
                    tg_user_id=tg_user_id,
                ))
                ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"] = task
        case EventTypeEnum.BotStop:
            task = ws_task.get(f"{tg_user_id}:{data.data.symbol.lower()}")
            
            # Отключаем клиента
            if task:
                task.cancel()
                del ws_task[f"{tg_user_id}:{data.data.symbol.lower()}"]

                message = MessageData(
                    event_data=MessageEventData(
                        type=EventDataTypeEnum.Log,
                        timestamp=data.timestamp,
                        payload=LogPayload(
                            event=data.event,
                            symbol=f"{data.data.symbol.upper()}",
                            longExchange=data.data.longExchange,
                            longOrderType=data.data.longOrderType,
                            shortExchange=data.data.shortExchange,
                            shortOrderType=data.data.shortOrderType,
                            status=LogStatusEnum.Success
                        )
                    ),
                    context=MessageContext(
                        method=MessageMethod.User,
                        tg_user_id=tg_user_id,
                    )
                )
                
                push_to_subscribes(message)

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
    
    try:
        if tg_user_id in user_state.exists_users():
            if user_state.long_size(tg_user_id) == 0:
                new_logs = await database.get_user_logs(tg_user_id)
                if not new_logs:
                    raise HTTPException(
                        status_code=404,
                        detail="Не удалось найти логов"
                    )

                user_state.set_logs(tg_user_id, new_logs)

                log.info("LogRouter -> Инициализация историй с базы данных с id")

                return ResultSchema(
                    status_code=200,
                    success=True,
                    message=LogMessageSchema(
                        logs=user_state.get_logs(tg_user_id)
                    )
                )
                
            else: 
                log.info("LogRouter -> Возращения данных из кеша")
                
                return ResultSchema(
                    status_code=200,
                    success=True,
                    message=LogMessageSchema(
                        logs=user_state.get_logs(tg_user_id)
                    )
                )
    except Exception as e:
        log.error(f"LogRouter -> {e}")

        return ResultSchema(
            status_code=404,
            success=False,
            message=LogMessageSchema(
                logs=[]
            )
        )