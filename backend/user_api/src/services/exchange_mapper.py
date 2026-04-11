import structlog

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