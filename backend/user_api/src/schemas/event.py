from enum import Enum
from typing import Optional, Union
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

class UserStatePayload(BaseModel):
    type: EventDataTypeEnum = EventDataTypeEnum.UserState
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
    payload: Union[MessageEventPayload, UserStatePayload]

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