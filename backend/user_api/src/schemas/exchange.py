from enum import Enum
from pydantic import BaseModel, Field

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class ExchangeEvent(str, Enum):
    AddExchange = "add_exchange"
    UpdateExchange = "update_exchange"

class ExchangePayload(BaseModel):
    event: ExchangeEvent
    exchange_name: ExchangeEnum
    is_available: True

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True