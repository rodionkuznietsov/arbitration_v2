import json
from urllib.parse import unquote

from fastapi import APIRouter, Request, HTTPException
import structlog
import hashlib
import hmac
import time

from ...db_schemas import ResultSchema, MessageSchema
from ...db import database
from ...tg_bot.app import BOT_TOKEN

log = structlog.get_logger()

router = APIRouter()

@router.post("/auth", tags=["telegram bot"])
async def auth_telegram(request: Request):
    data = await request.json()
    init_data = data.get("initData")

    if not init_data:
        raise HTTPException(status_code=400, detail="Требуется initData")
    
    is_valid = await verify_init_data(init_data)
    if not is_valid:
        raise HTTPException(status_code=401, detail="Пожалуйста войдите через телеграм")

    parsed_data = {k: unquote(v) for k, v in [s.split('=', 1) for s in init_data.split('&')]}
    auth_data = int(parsed_data.get("auth_date"))

    if time.time() - auth_data > 300: # 5 минут
        raise HTTPException(status_code=403, detail="Истёк срок годности")

    user = json.loads(parsed_data.get("user", "{}"))
    
    await database.connect()
    await database.add_user(user)
    await database.close()

    return ResultSchema(
        status_code=200,
        success=True,
        message=MessageSchema(
            tg_user_id=user.get("id")
        )
    )

async def verify_init_data(init_data: str):    
    try:
        vals = {k: unquote(v) for k, v in [s.split('=', 1) for s in init_data.split('&')]}
        data_check_string = '\n'.join(f"{k}={v}" for k, v in sorted(vals.items()) if k != 'hash')

        secret_key = hmac.new("WebAppData".encode(), BOT_TOKEN.encode(), hashlib.sha256).digest()
        h = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256)

        is_hash_valid = h.hexdigest() == vals.get('hash')

        return is_hash_valid
    except Exception as e: 
        log.error(e)
