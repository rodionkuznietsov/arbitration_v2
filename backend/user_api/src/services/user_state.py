import time
import structlog

from ..cache.exchange import available_exchanges
from ..schemas import AppStatusEnum, EventDataTypeEnum, ExchangeEnum, MessageContext, MessageData, MessageEventData, MessageMethod, OrderTypeEnum, UserStateEventTypeEnum, UserStateInitializationData, UserStatePayload, UserStateError

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
                        type=EventDataTypeEnum.UserState,
                        payload=UserStatePayload(
                            event=UserStateEventTypeEnum.InitData,
                            data=UserStateInitializationData(
                                isSleeping=AppStatusEnum.NotSleeping,
                                symbol="BTC",
                                longExchange=ExchangeEnum.Unknown,
                                longOrderType=OrderTypeEnum.Spot,

                                shortExchange=ExchangeEnum.Unknown,
                                shortOrderType=OrderTypeEnum.Spot,
                                logs=[]
                            )
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
    
    def update_exchange(
        self,
        tg_user_id: int,
        new_exchange: ExchangeEnum,
    ):
        if tg_user_id in self.__user_state__:
            log.info(available_exchanges)
            # Доработать
            if self.__user_state__[tg_user_id].event_data.payload.data.longExchange == new_exchange:
                self.__user_state__[tg_user_id].event_data.payload.data.longExchange = (
                    available_exchanges[0] if available_exchanges
                    else ExchangeEnum.Unknown
                ) 

            elif self.__user_state__[tg_user_id].event_data.payload.data.shortExchange == new_exchange:
                self.__user_state__[tg_user_id].event_data.payload.data.shortExchange = (
                    available_exchanges[1] if len(available_exchanges) > 1
                    else self.__user_state__[tg_user_id].event_data.payload.data.longExchange
                ) 

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

    def status(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.status

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
    
    def change_sleeping_status(
        self,
        tg_user_id: int,
        new_status: AppStatusEnum
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.isSleeping = new_status
        except Exception as e:
            log.error(f"{{user_state.change_sleeping_status}} -> {e}")