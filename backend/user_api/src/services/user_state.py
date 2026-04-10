import time
import structlog

from ..schemas import AppStatusEnum, EventTypeEnum, ExchangeEnum, MessageContext, MessageData, MessageEventData, MessageMethod, OrderTypeEnum, UserStatePayload, UserStateError

log: structlog.PrintLogger = structlog.get_logger()

class UserState:
    def __init__(self):
        self.__user_state__ = {}

    def push_default(
        self, 
        tg_user_id: int
    ):
        try:
            if tg_user_id not in self.__user_state__:
                self.__user_state__[tg_user_id] = MessageData(
                    event_data=MessageEventData(
                        type=EventTypeEnum.UserState,
                        payload=UserStatePayload(
                            symbol="BTC",
                            longExchange=ExchangeEnum.Bybit,
                            longOrderType=OrderTypeEnum.Spot,

                            shortExchange=ExchangeEnum.Gate,
                            shortOrderType=OrderTypeEnum.Spot,
                            logs=[]
                        ),
                        timestamp=int(time.time())
                    ),
                    context=MessageContext(
                        method=MessageMethod.User,
                        tg_user_id=tg_user_id
                    )
                )
        except Exception as e:
            log.error(f"UserState -> {e}")
    
    def get(
        self, 
        tg_user_id: int
    ):
        try:
            if tg_user_id in self.__user_state__:
                return self.__user_state__[tg_user_id]
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"UserState -> {e}")
    
    def isBotRunning(
        self,
        tg_user_id: int
    ):
        try:
            if tg_user_id in self.__user_state__:
                return self.__user_state__[tg_user_id].event_data.payload.isBotRunning
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"UserState(isBotRunning) -> {e}")

    def long_size(
        self,
        tg_user_id: int
    ):
        return len(self.__user_state__[tg_user_id].event_data.payload.logs)

    def get_logs(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.logs
        else:
            return []

    def set_logs(
        self,
        tg_user_id: int,
        logs: list[dict]
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.logs = logs
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"UserState(set_logs) -> {e}")

    def update_payload(
        self, 
        tg_user_id: int,
        symbol: str,
        longExchange: ExchangeEnum,
        longOrderType: OrderTypeEnum,
        shortExchange: ExchangeEnum,
        shortOrderType: OrderTypeEnum,
        status: AppStatusEnum,
        isBotRunning: AppStatusEnum
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.symbol = symbol
                self.__user_state__[tg_user_id].event_data.payload.longExchange = longExchange
                self.__user_state__[tg_user_id].event_data.payload.longOrderType = longOrderType
                self.__user_state__[tg_user_id].event_data.payload.shortExchange = shortExchange
                self.__user_state__[tg_user_id].event_data.payload.shortOrderType = shortOrderType
                self.__user_state__[tg_user_id].event_data.payload.status = status
                self.__user_state__[tg_user_id].event_data.payload.isBotRunning = isBotRunning
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"UserState -> {e}")

    def change_status(
        self,
        tg_user_id: int,
        status: AppStatusEnum,
        isBotRunning: AppStatusEnum
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.status = status
                self.__user_state__[tg_user_id].event_data.payload.isBotRunning = isBotRunning
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"UserState -> {e}")

    def get_users(self):
        return self.__user_state__.values()

    def exists_users(self):
        return self.__user_state__