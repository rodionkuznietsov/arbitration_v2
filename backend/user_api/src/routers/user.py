from typing import Annotated

from fastapi import APIRouter, Form, Depends

from ..schemas import ResultSchema, UserStateCmd
from ..jwt_func import oauth2_scheme
import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()

@router.post("/state/update", response_model=ResultSchema, tags=["user"])
async def update_user_state(
    data: UserStateCmd,
    token: Annotated[str, Depends(oauth2_scheme)]
):
    log.info(data)

    return ResultSchema(
        status_code=200,
        success=True,
        message="Команда получена"
    )