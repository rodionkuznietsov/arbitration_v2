from typing import Annotated

from fastapi import APIRouter, Form, Depends
from ..jwt_func import oauth2_scheme

router = APIRouter()

@router.post("/api/user/update", tags=["user"])
async def update_exchanges_keys(
    key: str = Form(...)
):
    return {
        "status": 200,
        "message": f"{key} added to user: 1",
    }

@router.get("/api/user/exchanges_keys", tags=["user"])
async def get_exchanges_keys(
    token: Annotated[str, Depends(oauth2_scheme)]
):
    return {
        "user": "Vitik1",
        "keys": {
            "api_key": "xxx",
            "api_secret": "xxx",
        } 
    }