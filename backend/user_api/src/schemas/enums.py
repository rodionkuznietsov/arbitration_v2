from enum import Enum

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"
    Binance = "binance"
    KuCoint = "kucoin"
    Unknown = "unknown"


class MarketTypeEnum(str, Enum):
    Long = "long"
    Short = "short"

class ExchangeEventEnum(str, Enum):
    AddExchange = "add_exchange"
    UpdateExchange = "update_exchange"
    ClearExchanges = "clear_exchanges"

class EventDataTypeEnum(str, Enum):
    Log = "log"
    Exchange = "exchange"
    Websocket = "websocket"
    UserState = "user_state"

class EventTypeEnum(str, Enum):
    UserState = "user_state"
    BotStart = "bot_start"
    BotStop = "bot_stop",
    Websocket = "websocket"


class UserStateEventTypeEnum(str, Enum):
    InitData = "init_data"
    ExchangeInvalidated = "exchange_invalidated"
    ExchangeUpdate = "exchange_update"
    SymbolUpdate = "symbol_update"