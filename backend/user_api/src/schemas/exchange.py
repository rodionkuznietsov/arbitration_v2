from enum import Enum
from pydantic import BaseModel, Field

class ExchangeEnum(str, Enum):
    Bybit = "bybit"
    Gate = "gate.io"

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True