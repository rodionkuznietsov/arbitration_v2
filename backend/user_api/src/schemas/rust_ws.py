from enum import Enum
from pydantic import BaseModel

from .enums import ExchangeEnum
from .bot import OrderTypeEnum


class WebSocketActionEnum(str, Enum):
    Subscribe = "subscribe"
    UnSubscribe = "unsubscribe"

class WebSocketChannelEnum(str, Enum):
    OrderBook = "order_book"
    Chart = "chart"

class WebSocketStatuEnum(str, Enum):
    Error = "error"
    Success = "success"
    
class WebsocketClosedContext(BaseModel):
    tg_user_id: int
    status: WebSocketStatuEnum

class WebsocketPayload(BaseModel):
    symbol: str
    longExchange: ExchangeEnum
    longOrderType: OrderTypeEnum
    shortExchange: ExchangeEnum
    shortOrderType: OrderTypeEnum