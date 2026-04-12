import time
from fastapi import APIRouter, HTTPException

from ..schemas import EventDataTypeEnum, ExchangeEventData, ExchangeMessageResponse, ExchangePayload, MarketTypeEnum, MessageContext, MessageData, ExchangeEventEnum, MessageEventData, MessageMethod, UserStatePayload, UserStateEventTypeEnum, UserStateUpdateData
from ..cache.exchange import available_exchanges, exchange_cache
from ..cache.cache import push_to_subscribes
from ..core.state import user_state

from ..db import database
from ..schemas.exchange import ExchangeSchema
from ..schemas.result import ResultSchema

import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()
@router.get("/available", tags=["exchanges"])
async def get_exchanges():
    return ResultSchema(
        status_code=200,
        success=True,
        message=ExchangeMessageResponse(
            exchanges=await exchange_cache.get()
        )
    )

@router.post("/add_exchange", response_model=ResultSchema, tags=["exchanges"])
async def add_exchange(exchange_data: ExchangeSchema):
    global available_exchanges
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
    global available_exchanges
    updated = await database.update_exchange_availability(exchange_data.name.lower(), exchange_data.is_available)
    if not updated:
        return ResultSchema(
            status_code=404,
            success=False,
            message=f"Биржа {exchange_data.name} не существует в базе данных."
        )
    
    exchange_cache.remove_or_insert(exchange_data=exchange_data)

    # Пушим событие юзерам, чтобы их ui обновился
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

    # Меняем данные для userState, чтобы навсякий случай избежать проблему с рассихроностью
    try:
        if not exchange_data.is_available:
            for tg_user_id in user_state.exists_users().keys():
                market_type = user_state.update_exchange(tg_user_id, exchange_data.name.lower())

                if market_type is not None:
                    # Пушим изменения для реального времени
                    message = MessageData(
                        event_data=MessageEventData(
                            type=EventDataTypeEnum.UserState,
                            timestamp=int(time.time()),
                            payload=UserStatePayload(
                                event=UserStateEventTypeEnum.ExchangeInvalidated,
                                data=UserStateUpdateData(
                                    exchange_name=exchange_data.name.lower(),
                                    fallback_exchange=exchange_cache.get_first_available_exchange(),
                                    market_type=market_type
                                )
                            )
                        ),
                        context=MessageContext(
                            method=MessageMethod.User,
                            tg_user_id=tg_user_id
                        )
                    )

                    push_to_subscribes(message)

    except Exception as e:
        log.error(f"{{ exchange_router.update_exchange_availability.user_state.exists_users }} -> {e}")

    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Статус доступности биржи {exchange_data.name} обновлен в базе данных."
    )

@router.delete('/delete_all_exchanges', response_model=ResultSchema, tags=["exchanges"])
async def delete_all_exchanges():
    await database.clear_table_exchanges()
    
    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Все биржи были удалены из базы данных"
    )