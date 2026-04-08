import asyncio
import json
from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException
import structlog

import jwt
from jwt.exceptions import InvalidTokenError, InvalidSubjectError

from ..schemas import EventTypeEnum

from .events import push_to_subscribes

from ..jwt_func import ALGORITHM, JWT_SECRET_KEY, oauth2_scheme

from ..db import database
from ..schemas import LogMessageSchema, ResultSchema, UserLogSchema

log: structlog.PrintLogger = structlog.get_logger()
router = APIRouter()

@router.post("/add/log", response_model=ResultSchema, tags=["logs"])
async def add_log(data: UserLogSchema, token: Annotated[str, Depends(oauth2_scheme)], ):
    global log_deque

    tg_user_id = int(authothicate(token))
    await database.add_log(tg_user_id, data)

    event_data = {
        "type": "log",
        "tg_user_id": tg_user_id, 
        "timestamp": data.timestamp,
        "payload": {
            "event": data.event, 
            "symbol": f"{data.data.symbol.upper()}USDT",
            "longExchange": data.data.long_exchange,
            "shortExchange": data.data.short_exchange,
            "isBotRunning": False
        }
    }
    
    match data.event:
        case EventTypeEnum.BotStart:
            log.info(f"{EventTypeEnum.BotStart}")
            event_data["payload"]["isBotRunning"] = True

    # Пушим на все устройства новое собитие
    await push_to_subscribes(event_data, tg_user_id)

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