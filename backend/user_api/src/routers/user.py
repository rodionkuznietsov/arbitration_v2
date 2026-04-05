from fastapi import APIRouter, Form

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
async def get_exchanges_keys():
    return {
        "user": "Vitik1",
        "keys": {
            "api_key": "xxx",
            "api_secret": "xxx",
        } 
    }