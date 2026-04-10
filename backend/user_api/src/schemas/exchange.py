from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field

from .enums import ExchangeEnum, EventDataTypeEnum, ExchangeEventEnum

class ExchangePayload(BaseModel):
    event: ExchangeEventEnum
    exchange_name: ExchangeEnum
    is_available: Optional[bool] = bool

class ExchangeEventData(BaseModel):
    type: EventDataTypeEnum
    timestamp: int
    payload: ExchangePayload

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True