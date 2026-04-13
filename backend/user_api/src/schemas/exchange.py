from enum import Enum
from typing import Optional, Union
from pydantic import BaseModel, Field

from .enums import ExchangeEnum, EventDataTypeEnum, ExchangeEventEnum

class ExchangeClearPayload(BaseModel):
    event: ExchangeEventEnum

class ExchangePayload(BaseModel):
    event: ExchangeEventEnum
    exchange_name: ExchangeEnum
    is_available: Optional[bool] = bool

class ExchangeEventData(BaseModel):
    type: EventDataTypeEnum
    timestamp: int
    payload: Union[ExchangePayload, ExchangeClearPayload]

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True