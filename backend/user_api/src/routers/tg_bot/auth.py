import json
from urllib.parse import unquote

from fastapi import APIRouter, Request, HTTPException
from ...db import database
from ...tg_bot.app import BOT_TOKEN
import hashlib
import hmac

router = APIRouter()

@router.post("/telegram", tags=["telegram bot auth"])
async def auth_telegram(request: Request):
    data = await request.json()
    init_data = data.get("initData")

    if not init_data:
        raise HTTPException(status_code=400, detail="Missing initData")

    is_valid = await verify_init_data(init_data)
    if not is_valid:
        raise HTTPException(status_code=401, detail="Invalid initData")

    parsed_data = {k: unquote(v) for k, v in [s.split('=', 1) for s in init_data.split('&')]}
    user = json.loads(parsed_data.get("user", "{}"))
    await database.connect()
    await database.add_user(user)
    await database.close()

    # Здесь создаём JWT токен для пользователя

    return {
        "status": 200,
        "message": "Login successful",
        "token": "fake-jwt-token"
    }   

async def verify_init_data(init_data: str):    
    vals = {k: unquote(v) for k, v in [s.split('=', 1) for s in init_data.split('&')]}
    data_check_string = '\n'.join(f"{k}={v}" for k, v in sorted(vals.items()) if k != 'hash')

    secret_key = hmac.new("WebAppData".encode(), BOT_TOKEN.encode(), hashlib.sha256).digest()
    h = hmac.new(secret_key, data_check_string.encode(), hashlib.sha256)

    is_valid = h.hexdigest() == vals.get('hash')
    return is_valid
