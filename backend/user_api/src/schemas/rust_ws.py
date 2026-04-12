from enum import Enum
from pydantic import BaseModel


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