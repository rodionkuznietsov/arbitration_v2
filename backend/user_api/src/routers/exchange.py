import time
from fastapi import APIRouter, HTTPException

from ..cache import user_state

from ..schemas import EventDataTypeEnum, ExchangeEventData, ExchangePayload, MessageData, ExchangeEventEnum
from ..cache import push_to_subscribes
from ..db import database
from ..schemas.exchange import ExchangeSchema
from ..schemas.result import ResultSchema

import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()
@router.get("/available", tags=["exchanges"])
async def get_exchanges():
    exchanges = await get_available_exchanges()
    return {
        "status": 200,
        "exchanges": exchanges
    }

async def get_available_exchanges():
    exchanges = await database.get_available_exchanges()
    return exchanges

@router.post("/add_exchange", response_model=ResultSchema, tags=["exchanges"])
async def add_exchange(exchange_data: ExchangeSchema):
    global exchange_cache
    added = await database.add_exchange(exchange_data.name.lower(), exchange_data.is_available)

    if not added:
        raise HTTPException(status_code=400, detail=f"Биржа {exchange_data.name} уже существует в базе данных.")

    message = MessageData(
        event_data=ExchangeEventData(
            type=EventDataTypeEnum.Exchange,
            payload=ExchangePayload(
                event=ExchangeEventEnum.AddExchange,
                exchange_name=exchange_data.name.lower(),
                is_available=exchange_data.is_available
            ),
            timestamp=int(time.time())
        )
    )

    push_to_subscribes(message)

    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Биржа {exchange_data.name} добавлена в базу данных."
    )

@router.put("/update_exchange_availability", response_model=ResultSchema, tags=["exchanges"])
async def update_exchange_availability(exchange_data: ExchangeSchema):
    global exchange_cache
    updated = await database.update_exchange_availability(exchange_data.name.lower(), exchange_data.is_available)
    if not updated:
        return ResultSchema(
            status_code=404,
            success=False,
            message=f"Биржа {exchange_data.name} не существует в базе данных."
        )

    # Меняем данные для userState, чтобы навсякий случай избежать проблему с рассихроностью
    try:
        for tg_user_id, user in user_state.exists_users().items():
            log.info(f"{tg_user_id} -> {user}")
    except Exception as e:
        log.error(f"{{ exchange_router.update_exchange_availability.user_state.exists_users }} -> {e}")

    message = MessageData(
        event_data=ExchangeEventData(
            type=EventDataTypeEnum.Exchange,
            payload=ExchangePayload(
                event=ExchangeEventEnum.UpdateExchange,
                exchange_name=exchange_data.name.lower(),
                is_available=exchange_data.is_available
            ),
            timestamp=int(time.time())
        )
    )
    
    push_to_subscribes(message)

    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Статус доступности биржи {exchange_data.name} обновлен в базе данных."
    )
    
@router.delete('/delete_all_exchanges', response_model=ResultSchema)
async def delete_all_exchanges():
    await database.clear_table_exchanges()
    
    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Все биржи были удалены из базы данных"
    )