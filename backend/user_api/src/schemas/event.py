from enum import Enum
from typing import Optional, Union
from pydantic import BaseModel

from ..schemas import ExchangeEnum
from .bot import AppStatusEnum, EventTypeEnum, LogStatusEnum, OrderTypeEnum

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

class LogPayload(BaseModel):
    event: EventTypeEnum
    symbol: str
    longExchange: ExchangeEnum
    longOrderType: OrderTypeEnum
    shortExchange: ExchangeEnum
    shortOrderType: OrderTypeEnum
    status: LogStatusEnum = AppStatusEnum.Running

class UserStatePayload(BaseModel):
    type: EventDataTypeEnum = EventDataTypeEnum.UserState,
    isSleeping: Optional[bool] = False
    symbol: Optional[str] = None
    longExchange: Optional[ExchangeEnum] = None
    longOrderType: Optional[OrderTypeEnum] = None
    shortExchange: Optional[ExchangeEnum] = None
    shortOrderType: Optional[OrderTypeEnum] = None
    status: Optional[AppStatusEnum] = AppStatusEnum.Offline
    isBotRunning: Optional[AppStatusEnum]= AppStatusEnum.Stopped
    logs: Optional[list] = []

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
    event_data: MessageEventData
    context: Optional[MessageContext] = None