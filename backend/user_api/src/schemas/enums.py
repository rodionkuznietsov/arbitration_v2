from enum import Enum


class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class EventDataTypeEnum(str, Enum):
    Log = "log"
    Exchange = "exchange"
    Websocket = "websocket"
    UserState = "user_state"