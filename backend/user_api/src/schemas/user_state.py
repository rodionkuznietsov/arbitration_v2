from typing import Optional, Union

from pydantic import BaseModel

from .enums import EventTypeEnum, ExchangeEnum, UserStateEventTypeEnum
from .event import AppStatusEnum, EventDataTypeEnum, OrderTypeEnum

class UserStateError(Exception):
    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(message)

class UserStateInitializationData(BaseModel):
    isSleeping: Optional[AppStatusEnum] = AppStatusEnum.Sleeping
    symbol: Optional[str] = None
    longExchange: Optional[ExchangeEnum] = None
    longOrderType: Optional[OrderTypeEnum] = None
    shortExchange: Optional[ExchangeEnum] = None
    shortOrderType: Optional[OrderTypeEnum] = None
    status: Optional[AppStatusEnum] = AppStatusEnum.Offline
    isBotRunning: Optional[bool] = False
    logs: Optional[list] = []

class UserStateUpdateData(BaseModel):
    exchange_name: ExchangeEnum
    fallback_exchange: ExchangeEnum

class UserStatePayload(BaseModel):
    event: UserStateEventTypeEnum
    data: Union[UserStateInitializationData, UserStateUpdateData]
