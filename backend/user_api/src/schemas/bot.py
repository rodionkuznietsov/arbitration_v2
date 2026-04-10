from enum import Enum
from typing import Optional
from pydantic import BaseModel
from ..schemas import ExchangeEnum

class EventTypeEnum(str, Enum):
    UserState = "user_state"
    BotStart = "bot_start"
    BotStop = "bot_stop",
    Websocket = "websocket"

class OrderTypeEnum(str, Enum):
    Spot = "спот"
    Futures = "фьючерс"

class AppStatusEnum(str, Enum):
    Online = "online",
    Offline = "offline"
    Warning = "warning"

    Running = True
    Stopped = False

class LogStatusEnum(str, Enum):
    Error = "Не удалось запустить бота",
    Success = "Бот был успешно запущен"

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
