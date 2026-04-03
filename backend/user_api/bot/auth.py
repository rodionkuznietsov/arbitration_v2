from fastapi import APIRouter, Request, HTTPException

router = APIRouter()

@router.post("/telegram")
async def auth_telegram(request: Request):
    data = await request.json()
    init_data = data.get("initData")

    if not init_data:
        raise HTTPException(status_code=400, detail="Missing initData")
    
    print("Received Telegram auth data:", init_data)

    return {
        "status": 200,
        "message": "Login successful",
        "token": "fake-jwt-token"
    }   