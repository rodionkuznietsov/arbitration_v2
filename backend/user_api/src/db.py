from time import time
from typing import Optional

import asyncpg
from dotenv import load_dotenv
import os
import structlog

from .schemas import UserLogSchema

log = structlog.get_logger()

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL")

class AsyncDatabase:
    pool: Optional[asyncpg.Connection] = None

    async def connect(self, min_size, max_size):
        try:
            if self.pool is None:
                self.pool = await asyncpg.create_pool(
                    dsn=DATABASE_URL,
                    min_size=min_size,
                    max_size=max_size
                )
        except Exception as e:
            log.error(f"AsyncDatabase -> {e}")
        log.info("Connected to the database.")

    async def close(self):
        if self.pool:
            await self.pool.close()
            self.pool = None
            log.info("Database connection closed.")

    async def add_user(self, user_data: dict):
        if await self.__check_user_exists__(user_data.get("id")):
            log.warning(f"Пользователь {user_data.get('id')} уже существует в базе данных.")
            return False
        
        # Добавляем пользователя в базу данных 
        await self.pool.execute(
            "INSERT INTO users (tg_user_id) VALUES ($1)",
            user_data.get("id")
        )
        log.info(f"Пользователь {user_data.get('id')} добавлен в базу данных.")
        return True

    async def __check_user_exists__(self, user_id: int) -> bool:
        result = await self.pool.fetchrow(
            "SELECT id FROM users WHERE tg_user_id = $1",
            user_id
        )
        return result is not None
    
    async def get_available_exchanges(self) -> list:
        try:
            result = await self.pool.fetch(
                "SELECT * FROM exchanges WHERE is_available = true ORDER BY id ASC"
            )
        except Exception as e: 
            log.error(f"AsyncDatabase -> {e}")
            result = None
        return result
    
    async def add_exchange(self, exchange_name: str, is_available: bool) -> bool:
        if await self.__check_exchange_exists__(exchange_name):
            log.warning(f"Биржа {exchange_name} уже существует в базе данных.")
            return False

        await self.pool.execute(
            "INSERT INTO exchanges (name, is_available) VALUES ($1, $2)",
            exchange_name, is_available
        )
        log.info(f"Биржа {exchange_name} добавлена в базу данных.")
        return True
    
    async def update_exchange_availability(self, exchange_name: str, is_available: bool) -> bool:
        if not await self.__check_exchange_exists__(exchange_name):
            log.warning(f"Биржа {exchange_name} не существует в базе данных.")
            return False

        await self.pool.execute(
            "UPDATE exchanges SET is_available = $1 WHERE name = $2",
            is_available, exchange_name
        )

        log.info(f"Статус доступности биржи {exchange_name} обновлен в базе данных.")
        return True
    
    async def __check_exchange_exists__(self, exchange_name: str) -> bool:
        result = await self.pool.fetchrow(
            "SELECT id FROM exchanges WHERE name = $1",
            exchange_name
        )
        return result is not None
    
    async def add_log(self, tg_user_id: int, data: UserLogSchema):
        if await self.__check_log_exists__(data.timestamp, tg_user_id):
            log.warning(f"Не удалось добавить лог")
            return
        
        await self.pool.execute(
            "INSERT INTO user_logs (event, tg_user_id, symbol, long_exchange, short_exchange, timestamp) VALUES ($1, $2, $3, $4, $5, $6)",
            data.event, tg_user_id, data.data.symbol, data.data.long_exchange, data.data.short_exchange, data.timestamp
        )
        log.info(f"Лог: {data.event.title()}, успешно был добавлен для пользователя с id: {tg_user_id}.")

    async def __check_log_exists__(self, timestamp, tg_user_id):
        if timestamp is not None:
            result = await self.pool.fetchrow(
                "SELECT timestamp FROM user_logs WHERE timestamp = $1 AND tg_user_id = $2",
                timestamp, tg_user_id
            )
        else:
            result = await self.pool.fetchrow(
                "SELECT timestamp FROM user_logs WHERE tg_user_id = $1",
                tg_user_id
            )

        return result is not None

    async def get_user_logs(self, tg_user_id: int):
        if not await self.__check_log_exists__(None, tg_user_id):
            log.warning(f"Не удалось найти логов для пользователя с id: {tg_user_id}")
            return
        
        result = await self.pool.fetch(
            "SELECT * FROM user_logs WHERE tg_user_id = $1 ORDER BY timestamp DESC",
            tg_user_id
        )

        list_of_dicts = [dict(record) for record in result]
        return list_of_dicts

    async def clear_table_user_logs(self):
        await self.pool.execute(
            "TRUNCATE TABLE user_logs"
        )
        log.info("Таблица: user_logs, была очищена.")

database = AsyncDatabase()

