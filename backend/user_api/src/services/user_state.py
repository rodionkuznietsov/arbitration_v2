from pyclbr import Class
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
            log.error(f"UserState(get) -> {e}")
    
    def isBotRunning(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.isBotRunning

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
        except Exception as e:
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
        except Exception as e:
            log.error(f"UserState(update_payload) -> {e}")

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
        except Exception as e:
            log.error(f"UserState(ChangeStatus) -> {e}")

    def get_users(self):
        return self.__user_state__.values()

    def exists_users(self):
        return self.__user_state__
    
class SafeUserState(UserState):
    def __init__(self):
        super().__init__()

    def change_status(self, tg_user_id, status, isBotRunning):
        try:
            return super().change_status(tg_user_id, status, isBotRunning)
        except Exception as e:
            log.error(f"UserState -> {e}")

    def push_default(self, tg_user_id):
        return super().push_default(tg_user_id)
    
    def get(self, tg_user_id):
        return super().get(tg_user_id)
    
    def get_logs(self, tg_user_id):
        return super().get_logs(tg_user_id)
    
    def get_users(self):
        return super().get_users()
    
    def exists_users(self):
        return super().exists_users()
    
    def update_payload(self, tg_user_id, symbol, longExchange, longOrderType, shortExchange, shortOrderType, status, isBotRunning):
        return super().update_payload(tg_user_id, symbol, longExchange, longOrderType, shortExchange, shortOrderType, status, isBotRunning)
    
    def isBotRunning(self, tg_user_id):
        return super().isBotRunning(tg_user_id)
    
    def set_logs(self, tg_user_id, logs):
        return super().set_logs(tg_user_id, logs)
    
    def long_size(self, tg_user_id):
        return super().long_size(tg_user_id)
    