from pydantic import BaseModel, Field

class ExchangeSchema(BaseModel):
    name: str = Field(..., example="Binance")
    is_available: bool = True