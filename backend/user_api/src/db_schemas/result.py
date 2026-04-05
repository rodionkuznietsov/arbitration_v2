from pydantic import BaseModel

class ResultSchema(BaseModel):
    status_code: int
    success: bool
    message: str