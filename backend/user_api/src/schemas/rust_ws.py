from enum import Enum

class WebSocketActionEnum(str, Enum):
    Subscribe = "subscribe"
    UnSubscribe = "unsubscribe"

class WebSocketChannelEnum(str, Enum):
    OrderBook = "order_book"
    Chart = "chart"