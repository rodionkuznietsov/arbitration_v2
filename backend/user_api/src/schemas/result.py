from pydantic import BaseModel
from .access_token import TokenSchema
    
class LogMessageSchema(BaseModel):
    logs: list

class MessageSchema(BaseModel):
    tg_user_id: int
    token_data: TokenSchema

class ExchangeMessageResponse(BaseModel):
    exchanges: dict

class ResultSchema(BaseModel):
    status_code: int
    success: bool
    message: str | TokenSchema | LogMessageSchema | MessageSchema | ExchangeMessageResponse