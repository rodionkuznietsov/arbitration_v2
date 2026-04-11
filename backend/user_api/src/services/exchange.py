import structlog

from ..schemas import ExchangeSchema

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
    try:
        if exchange_data.is_available:
            pass
        #     available_exchanges[exchange_data.name] = exchange_data.is_available
        # else: 
        #     available_exchanges.pop(exchange_data.name)

        # log.info(f"Exchanges: {available_exchanges}")
    except Exception as e:
        log.error(f"{{ exchange_router.update_exchange_availability.is_available }} -> {e}")