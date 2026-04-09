from enum import Enum


class EventDataTypeEnum(str, Enum):
    Log = "log"
    Exchange = "exchange"
    Websocket = "websocket"