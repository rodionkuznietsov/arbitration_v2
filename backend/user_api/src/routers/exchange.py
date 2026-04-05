from fastapi import APIRouter
import asyncio

from ..db import database
from ..db_schemas.exchange import ExchangeSchema
from ..db_schemas.result import ResultSchema

exchange_cache = []

router = APIRouter()
@router.get("/available", tags=["exchanges"])
async def get_exchanges():
    global exchange_cache
    return {
        "status": 200,
        "exchanges": exchange_cache
    }

async def get_available_exchanges():
    await database.connect()
    exchanges = await database.get_available_exchanges()
    await database.close()
    return exchanges

@router.post("/add_exchange", response_model=ResultSchema, tags=["exchanges"])
async def add_exchange(exchange_data: ExchangeSchema):
    global exchange_cache
    await database.connect()
    added = await database.add_exchange(exchange_data.name, exchange_data.is_available)
    await database.close()
    exchange_cache = await get_available_exchanges()  # Обновляем кэш после добавления новой биржи

    if not added:
        return ResultSchema(
            status_code=400,
            success=False,
            message=f"Биржа {exchange_data.name} уже существует в базе данных."
        )
    
    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Биржа {exchange_data.name} добавлена в базу данных."
    )

@router.put("/update_exchange_availability", response_model=ResultSchema, tags=["exchanges"])
async def update_exchange_availability(exchange_data: ExchangeSchema):
    global exchange_cache
    await database.connect()
    updated = await database.update_exchange_availability(exchange_data.name, exchange_data.is_available)
    await database.close()
    if not updated:
        return ResultSchema(
            status_code=404,
            success=False,
            message=f"Биржа {exchange_data.name} не существует в базе данных."
        )
    
    exchange_cache = await get_available_exchanges()  # Обновляем кэш после изменения статуса доступности
    return ResultSchema(
        status_code=200,
        success=True,
        message=f"Статус доступности биржи {exchange_data.name} обновлен в базе данных."
    )
    

async def refresh_exchanges_availability():
    global exchange_cache
    while True:
        exchange_cache = await get_available_exchanges()
        await asyncio.sleep(60)  # Обновляем кэш каждые 60 секунд