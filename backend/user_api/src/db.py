from time import time

import asyncpg
from dotenv import load_dotenv
import os
import structlog

from .db_schemas import UserLogSchema

log = structlog.get_logger()

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL")

class AsyncDatabase:
    def __init__(self):
        self.conn = None

    async def connect(self):
        self.conn = await asyncpg.connect(DATABASE_URL)
        log.info("Connected to the database.")

    async def close(self):
        if self.conn:
            await self.conn.close()
            log.info("Database connection closed.")

    async def add_user(self, user_data: dict):
        if await self.__check_user_exists__(user_data.get("id")):
            log.info(f"Пользователь {user_data.get('id')} уже существует в базе данных.")
            return 
        
        # Добавляем пользователя в базу данных 
        await self.conn.execute(
            "INSERT INTO users (tg_user_id) VALUES ($1)",
            user_data.get("id")
        )
        log.info(f"Пользователь {user_data.get('id')} добавлен в базу данных.")

    async def __check_user_exists__(self, user_id: int) -> bool:
        result = await self.conn.fetchrow(
            "SELECT id FROM users WHERE tg_user_id = $1",
            user_id
        )
        return result is not None
    
    async def get_available_exchanges(self) -> list:
        result = await self.conn.fetch(
            "SELECT * FROM exchanges WHERE is_available = true ORDER BY id ASC"
        )
        return result

    async def add_exchange(self, exchange_name: str, is_available: bool) -> bool:
        if await self.__check_exchange_exists__(exchange_name):
            log.info(f"Биржа {exchange_name} уже существует в базе данных.")
            return False

        await self.conn.execute(
            "INSERT INTO exchanges (name, is_available) VALUES ($1, $2)",
            exchange_name, is_available
        )
        log.info(f"Биржа {exchange_name} добавлена в базу данных.")
        return True
    
    async def update_exchange_availability(self, exchange_name: str, is_available: bool) -> bool:
        if not await self.__check_exchange_exists__(exchange_name):
            log.info(f"Биржа {exchange_name} не существует в базе данных.")
            return False

        await self.conn.execute(
            "UPDATE exchanges SET is_available = $1 WHERE name = $2",
            is_available, exchange_name
        )
        log.info(f"Статус доступности биржи {exchange_name} обновлен в базе данных.")
        return True
    
    async def __check_exchange_exists__(self, exchange_name: str) -> bool:
        result = await self.conn.fetchrow(
            "SELECT id FROM exchanges WHERE name = $1",
            exchange_name
        )
        return result is not None
    
    async def add_log(self, data: UserLogSchema):
        if await self.__check_log_exists__(data.timestamp, data.data.tg_user_id):
            log.warning(f"Не удалось добавить лог")
            return
        
        await self.conn.execute(
            "INSERT INTO user_logs (event, tg_user_id, symbol, long_exchange, short_exchange, timestamp) VALUES ($1, $2, $3, $4, $5, $6)",
            data.event, data.data.tg_user_id, data.data.symbol, data.data.long_exchange, data.data.short_exchange, data.timestamp
        )
        log.warning(f"Лог {data.event} успешно был добавлен для пользователя с id: {data.data.tg_user_id}.")

    async def __check_log_exists__(self, timestamp, tg_user_id):
        result = await self.conn.fetchrow(
            "SELECT timestamp FROM user_logs WHERE timestamp = $1 AND tg_user_id = $2",
            timestamp, tg_user_id
        )
        return result is not None

database = AsyncDatabase()