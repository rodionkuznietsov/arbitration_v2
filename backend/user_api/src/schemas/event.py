from enum import Enum
from typing import Optional
from pydantic import BaseModel

from ..schemas import ExchangeEnum
from .bot import AppStatusEnum, EventTypeEnum, OrderTypeEnum

class EventDataTypeEnum(str, Enum):
    Log = "log"
    Exchange = "exchange"
    Websocket = "websocket"
    UserState = "user_state"

class MessageEventPayload(BaseModel):
    event: EventTypeEnum
    symbol: str
    longExchange: ExchangeEnum
    longOrderType: OrderTypeEnum
    shortExchange: ExchangeEnum
    shortOrderType: OrderTypeEnum
    status: AppStatusEnum = AppStatusEnum.Offline
    isBotRunning: AppStatusEnum = AppStatusEnum.Stopped

class MessageEventData(BaseModel):
    type: EventDataTypeEnum
    timestamp: int
    payload: MessageEventPayload

class MessageMethod(str, Enum):
    User = "user"
    WebsocketConnected = "websocket_connected"
    WebsocketErrorConnection = "websocket_error_connection"

class MessageContext(BaseModel):
    method: MessageMethod
    tg_user_id: int

class MessageData(BaseModel):
    event_data: MessageEventData
    context: Optional[MessageContext] = None