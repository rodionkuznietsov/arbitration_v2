from typing import Annotated

from fastapi import APIRouter, Form, Depends

from ..services import authothicate

from ..core.state import user_state

from ..schemas import ResultSchema, UserStateCmd, UserStateEventTypeEnum
from ..jwt_func import oauth2_scheme
import structlog

log: structlog.PrintLogger = structlog.get_logger()

router = APIRouter()

@router.post("/state/update", response_model=ResultSchema, tags=["user"])
async def update_user_state(
    data: UserStateCmd,
    token: Annotated[str, Depends(oauth2_scheme)]
):
    
    tg_user_id = int(authothicate(token))

    # try:
    #     if data.event == UserStateEventTypeEnum.ExchangeUpdate:
    #         user_state.update_draft_exchange(tg_user_id, data.data.exchange_name, data.data.market_type)
    #         log.info(f"{{ user_router.update_user_state.update_exchange }}")
    # except Exception as e:
    #     log.error(f"{{ user_router.update_user_state.update_exchange }} -> {e}")

    # try:
    #     if data.event == UserStateEventTypeEnum.SymbolUpdate:
    #         user_state.update_draft_symbol(tg_user_id, data.data.symbol)
    #         log.info(f"{{ user_router.update_user_state.update_exchange }}")
    # except Exception as e:
    #     log.error(f"{{ user_router.update_user_state.update_exchange }} -> {e}")

    return ResultSchema(
        status_code=200,
        success=True,
        message="Команда получена"
    )