from typing import Annotated

from fastapi import APIRouter, Form, Depends

from ..schemas import UserStateCmd
from ..jwt_func import oauth2_scheme
import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()

@router.post("/state/update", tags=["user"])
async def update_user_state(
    data: UserStateCmd,
    token: Annotated[str, Depends(oauth2_scheme)]
):
    log.info(data)
