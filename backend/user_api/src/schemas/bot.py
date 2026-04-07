from enum import Enum
from pydantic import BaseModel

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class EventTypeEnum(str, Enum):
    BotStart = "bot_start"
    BotStop = "bot_stop"

class LogDataSchema(BaseModel):
    symbol: str
    long_exchange: ExchangeEnum
    short_exchange: ExchangeEnum

class UserLogSchema(BaseModel):
    event: EventTypeEnum
    data: LogDataSchema
    timestamp: int
