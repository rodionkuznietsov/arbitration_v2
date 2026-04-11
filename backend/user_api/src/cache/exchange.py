from ..db import AsyncDatabase, database
from ..schemas import ExchangeSchema
available_exchanges = {}

import structlog
log: structlog.PrintLogger = structlog.get_logger()

class ExchangeCache():
    def __init__(self, database: AsyncDatabase):
        self.__available_exchanges__ = {}
        self.__database__ = database

    def exchange_mapper(
        self,
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

    async def get(self):
        try:
            if exchange_cache.get_size() == 0:
                log.info(f"{{ get_available_exchanges_service.database }}")
                raw_exchanges = await self.__database__.get_available_exchanges()
                mapped = self.exchange_mapper(raw_exchanges)

                exchange_cache.set_data(mapped)
            else:
                log.info(f"{{ get_available_exchanges_service.cache }}")
        except Exception as e:
            log.error(f"{{ exchange_service.get_available_exchanges_service }} -> {e}")
    
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
                if exchange_name in self.__available_exchanges__:
                    self.__available_exchanges__.pop(exchange_name)        
        except Exception as e:
            log.error(f"{{ exchange_cache.update_available_exchanges_in_cache }} -> {e}")

exchange_cache = ExchangeCache(database=database)