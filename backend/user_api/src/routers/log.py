from fastapi import APIRouter, Body
import structlog

from ..db import database
from ..db_schemas import UserLogSchema

log = structlog.get_logger()
router = APIRouter()

@router.post("/log", tags=["logs"])
async def bot_start(data: UserLogSchema):
    await database.connect()
    await database.add_log(data)
    await database.close()