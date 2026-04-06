from pydantic import BaseModel

class MessageSchema(BaseModel):
    tg_user_id: int

class ResultSchema(BaseModel):
    status_code: int
    success: bool
    message: str | MessageSchema