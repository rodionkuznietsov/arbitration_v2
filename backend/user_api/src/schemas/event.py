from enum import Enum
from typing import Optional, Union
from pydantic import BaseModel

from ..schemas import ExchangeEnum, ExchangeEventData
from .bot import AppStatusEnum, EventTypeEnum, LogStatusEnum, OrderTypeEnum
from .enums import EventDataTypeEnum, ExchangeEnum
from .user_state import UserStatePayload

class MessageEventPayload(BaseModel):
    event: EventTypeEnum
    symbol: str
    longExchange: ExchangeEnum
    longOrderType: OrderTypeEnum
    shortExchange: ExchangeEnum
    shortOrderType: OrderTypeEnum
    status: AppStatusEnum = AppStatusEnum.Offline
    isBotRunning: Optional[bool] = False

class LogPayload(BaseModel):
    event: EventTypeEnum
    symbol: str
    longExchange: ExchangeEnum
    longOrderType: OrderTypeEnum
    shortExchange: ExchangeEnum
    shortOrderType: OrderTypeEnum
    status: LogStatusEnum = LogStatusEnum.Success

class MessageMethod(str, Enum):
    User = "user"
    WebsocketConnected = "websocket_connected"
    WebsocketErrorConnection = "websocket_error_connection"
    WebsocketClosed = "websocket_closed"

class MessageWebsocketData(BaseModel):
    method: MessageMethod
    tg_user_id: int

class MessageEventData(BaseModel):
    type: EventDataTypeEnum
    timestamp: int
    payload: Union[MessageEventPayload, UserStatePayload, LogPayload] = None
    ws_data: Optional[dict] = None

class MessageContext(BaseModel):
    method: MessageMethod
    tg_user_id: int

class MessageData(BaseModel):
    event_data: Union[MessageEventData, ExchangeEventData]
    context: Optional[MessageContext] = None