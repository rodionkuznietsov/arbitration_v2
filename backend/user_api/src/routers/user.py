from typing import Annotated

from fastapi import APIRouter, Form, Depends

from ..schemas import UserStateCmd
from ..jwt_func import oauth2_scheme
import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()

@router.post("/user/update", tags=["user"])
async def update_exchanges_keys(
    data: UserStateCmd
):
    log.info(data)
