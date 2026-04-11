from ..schemas import ExchangeSchema
available_exchanges = {}

import structlog

log: structlog.PrintLogger = structlog.get_logger()

class ExchangeCache:
    def __init__(self):
        self.__available_exchanges__ = {}

    def get(self):
        return self.__available_exchanges__
    
    def get_size(self):
        return len(self.__available_exchanges__)

    def set_data(
        self,
        data,
    ):
        self.__available_exchanges__ = data

    def remove_or_insert(
        self,
        exchange_data: ExchangeSchema
    ):
        try:
            exchange_name: str = exchange_data.name.lower()
            if exchange_data.is_available:
                self.__available_exchanges__[exchange_name] = exchange_data.is_available
            else: 
                if exchange_name in available_exchanges:
                    self.__available_exchanges__.pop(exchange_name)        
        except Exception as e:
            log.error(f"{{ exchange_cache.update_available_exchanges_in_cache }} -> {e}")

exchange_cache = ExchangeCache()