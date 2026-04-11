from re import L

import structlog
from ..schemas import ExchangeSchema
from ..cache.exchange import exchange_cache
from ..db import database

log: structlog.PrintLogger = structlog.get_logger()

def exchange_mapper(
    raw_exchanges: list
):
    try:
        mapped = {}
        for raw_exchange in raw_exchanges:
            name = raw_exchange["name"].lower()
            mapped[name] = raw_exchange["is_available"]

        return mapped
    except Exception as e:
        log.error(f"{{ exchange_mapper }}  -> {e}")

def update_available_exchanges_in_cache(
    exchange_data: ExchangeSchema
):
    global available_exchanges

    try:
        if exchange_data.is_available:
            available_exchanges[exchange_data.name.lower()] = exchange_data.is_available
        else: 
            if exchange_data.name.lower() in available_exchanges:
                available_exchanges.pop(exchange_data.name.lower())        
    except Exception as e:
        log.error(f"{{ update_available_exchanges_in_cache }} -> {e}")

async def get_available_exchanges_service():
    if len(exchange_cache.get_size()) == 0:
        log.info(f"{{ get_available_exchanges_service.database }}")
        raw_exchanges = await database.get_available_exchanges()
        mapped = exchange_mapper(raw_exchanges)

        exchange_cache.set_data(mapped)
    else:
        log.info(f"{{ get_available_exchanges_service.cache }}")
