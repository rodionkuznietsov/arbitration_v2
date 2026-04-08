from enum import Enum
from pydantic import BaseModel

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class EventTypeEnum(str, Enum):
    BotStart = "bot_start"
    BotStop = "bot_stop"

class OrderTypeEnum(str, Enum):
    Spot = "спот",
    Futures = "фьючерс"

class LogDataSchema(BaseModel):
    symbol: str
    
    long_exchange: ExchangeEnum
    long_order_type: OrderTypeEnum

    short_exchange: ExchangeEnum
    short_order_type: OrderTypeEnum

class UserLogSchema(BaseModel):
    event: EventTypeEnum
    data: LogDataSchema
    timestamp: int
