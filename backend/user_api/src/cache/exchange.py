available_exchanges = {}

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
    
    def remove(
        self,
        exchange_name: str
    ):
        self.__available_exchanges__.pop(exchange_name)

exchange_cache = ExchangeCache()