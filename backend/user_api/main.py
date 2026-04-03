import asyncio

from fastapi import FastAPI, Form
from fastapi.middleware.cors import CORSMiddleware
from bot.auth import router as auth_router
from bot.app import app as bot_app

app = FastAPI()
app.include_router(auth_router)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://192.168.179.78:8080", "https://unfarming-untethered-flynn.ngrok-free.dev"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"]
)

@app.on_event("startup")
async def startup():
    await bot_app.initialize()
    await bot_app.start()
    asyncio.create_task(bot_app.updater.start_polling())

@app.on_event("shutdown")
async def shutdown():
    await bot_app.stop()

@app.get("/", include_in_schema=False)
async def root():
    return { 
        "status": 200,
        "version": 1
    }   

@app.post("/user/update")
async def update_exchanges_keys(
    key: str = Form(...)
):
    return {
        "status": 200,
        "message": f"{key} added to user: 1",
    }

@app.get("/user/exchanges_keys")
async def get_exchanges_keys():
    return {
        "user": "Vitik1",
        "keys": {
            "api_key": "xxx",
            "api_secret": "xxx",
        } 
    }