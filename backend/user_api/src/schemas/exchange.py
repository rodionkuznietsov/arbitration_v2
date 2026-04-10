from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field

from .enums import ExchangeEnum, EventDataTypeEnum

class ExchangeEvent(str, Enum):
    AddExchange = "add_exchange"
    UpdateExchange = "update_exchange"

class ExchangePayload(BaseModel):
    event: ExchangeEvent
    exchange_name: ExchangeEnum
    is_available: Optional[bool] = bool

class ExchangeEventData(BaseModel):
    type: EventDataTypeEnum
    timestamp: int
    payload: ExchangePayload

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True