from pydantic import BaseModel

class MessageSchema(BaseModel):
    tg_user_id: int

class LogMessageSchema(BaseModel):
    logs: list

class ResultSchema(BaseModel):
    status_code: int
    success: bool
    message: str | MessageSchema | LogMessageSchema