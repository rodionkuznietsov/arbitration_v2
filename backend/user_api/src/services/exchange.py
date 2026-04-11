import structlog
from ..schemas import ExchangeSchema
from ..cache.exchange import available_exchanges
from ..db import database

log: structlog.PrintLogger = structlog.get_logger()

def exchange_mapper(
    raw_exchanges: list
):
    try:
        mapped = {}
        for raw_exchange in raw_exchanges:
            mapped[raw_exchange["name"]] = raw_exchange["is_available"]

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
                available_exchanges.pop(exchange_data.name)        
    except Exception as e:
        log.error(f"{{ update_available_exchanges_in_cache }} -> {e}")

async def get_available_exchanges_service():
    global available_exchanges

    if len(available_exchanges) == 0:
        log.info(f"{{ get_available_exchanges_service.database }}")
        raw_exchanges = await database.get_available_exchanges()
        available_exchanges = exchange_mapper(raw_exchanges)
    else:
        log.info(f"{{ get_available_exchanges_service.cache }}")

    return available_exchanges