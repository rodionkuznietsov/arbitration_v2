from enum import Enum


class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"
    Binance = "binance"
    Unknown = "unknown"

class ExchangeEventEnum(str, Enum):
    AddExchange = "add_exchange"
    UpdateExchange = "update_exchange"


class EventDataTypeEnum(str, Enum):
    Log = "log"
    Exchange = "exchange"
    Websocket = "websocket"
    UserState = "user_state"