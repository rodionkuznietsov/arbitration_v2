from enum import Enum
from typing import Optional
from pydantic import BaseModel

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class EventTypeEnum(str, Enum):
    BotStart = "bot_start"
    BotStop = "bot_stop"

class OrderTypeEnum(str, Enum):
    Spot = "спот"
    Futures = "фьючерс"

class AppStatusEnum(str, Enum):
    Online = "online",
    Offline = "offline"

class LogDataSchema(BaseModel):
    symbol: str

    longExchange: ExchangeEnum
    longOrderType: Optional[OrderTypeEnum] = None

    shortExchange: ExchangeEnum
    shortOrderType: Optional[OrderTypeEnum] = None

class UserLogSchema(BaseModel):
    event: EventTypeEnum
    data: LogDataSchema
    timestamp: int
