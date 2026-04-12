from pydantic import BaseModel

from .enums import EventTypeEnum, ExchangeEnum, UserStateEventTypeEnum
from .event import EventDataTypeEnum

class UserStateError(Exception):
    def __init__(self, status_code: int, message: str):
        self.status_code = status_code
        self.message = message
        super().__init__(message)

class UserStateEventPayload(BaseModel):
    event: UserStateEventTypeEnum
    exchange_name: ExchangeEnum
    fallback_exchange: ExchangeEnum