import time
import structlog

from ..cache.exchange import exchange_cache
from ..schemas import AppStatusEnum, BotConfig, BotConfigData, EventDataTypeEnum, ExchangeEnum, MarketTypeEnum, MessageContext, MessageData, MessageEventData, MessageMethod, OrderTypeEnum, UserStateEventTypeEnum, UserStateInitializationData, UserStatePayload, UserStateError

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
                log.info(f"{{ user_state.push_default }} -> {tg_user_id}")

                self.__user_state__[tg_user_id] = MessageData(
                    event_data=MessageEventData(
                        type=EventDataTypeEnum.UserState,
                        payload=UserStatePayload(
                            event=UserStateEventTypeEnum.InitData,
                            data=UserStateInitializationData(
                                isSleeping=AppStatusEnum.NotSleeping,
                                status=AppStatusEnum.Offline,
                                isBotRunning=False,
                                logs=[]
                            ),
                            bot_config=BotConfig(
                                active=BotConfigData(),
                                draft=BotConfigData(
                                    symbol="BTC",
                                    longExchange=ExchangeEnum.Unknown,
                                    longOrderType=OrderTypeEnum.Spot,
                                    shortExchange=ExchangeEnum.Unknown,
                                    shortOrderType=OrderTypeEnum.Spot,
                                )
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
    
    def __is_bot_running__(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].payload.data.isBotRunning

    def get_active_config(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active

    def set_active_from_draft(
        self,
        tg_user_id: int
    ): 
        self.__user_state__[tg_user_id].event_data.payload.bot_config.active = self.__user_state__[tg_user_id].event_data.payload.bot_config.draft
        log.info(f"{{ user_state.set_active_from_draft }} -> успешно")

    def update_draft_exchange(
        self,
        tg_user_id: int,
        new_exchange: ExchangeEnum,
        market_type: MarketTypeEnum
    ):
        if market_type == MarketTypeEnum.Long:
            self.__user_state__[tg_user_id].event_data.payload.bot_config.draft.longExchange = new_exchange
        elif market_type == MarketTypeEnum.Short:
            self.__user_state__[tg_user_id].event_data.payload.bot_config.draft.shortExchange = new_exchange
        else:
            raise Exception("Неизвестный тип market_type")

    def update_draft_symbol(
        self,
        tg_user_id,
        new_symbol
    ):
        self.__user_state__[tg_user_id].event_data.payload.bot_config.draft.symbol = new_symbol

    def update_exchange_invalidated(
        self,
        tg_user_id: int,
        new_exchange: ExchangeEnum,
    ):
        """Делает проверку на выбранную ими биржу, если она являеться не валидной, заменяет её на первую или последнюю доступную"""

        if tg_user_id in self.__user_state__:
            types = []
            
            if self.__user_state__[tg_user_id].event_data.payload.data.longExchange == new_exchange:
                self.__user_state__[tg_user_id].event_data.payload.data.longExchange = exchange_cache.get_first_available_exchange()
                types.append(MarketTypeEnum.Long)

            if self.__user_state__[tg_user_id].event_data.payload.data.shortExchange == new_exchange:
                self.__user_state__[tg_user_id].event_data.payload.data.shortExchange = exchange_cache.get_last_available_exchange()
                types.append(MarketTypeEnum.Short)

            return types
        
    def get(
        self, 
        tg_user_id: int
    ):
        try:
            if tg_user_id in self.__user_state__:
                if self.__is_bot_running__(tg_user_id):
                    self.__user_state__[tg_user_id].event_data.payload.bot_config.draft = self.__user_state__[tg_user_id].event_data.payload.bot_config.active
                    
                return self.__user_state__[tg_user_id]
        
            raise UserStateError(status_code=404, message=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(f"{{ user_state.get }} -> {e}")
    
    def long_active_symbol(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active.symbol

    def long_active_exchange(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active.longExchange

    def long_active_order_type(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active.longOrderType

    def short_active_exchange(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active.shortExchange

    def short_active_order_type(
        self,
        tg_user_id: int
    ):
        return self.__user_state__[tg_user_id].event_data.payload.bot_config.active.shortOrderType

    def isBotRunning(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.data.isBotRunning

    def status(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.data.status

    def long_size(
        self,
        tg_user_id: int
    ):
        return len(self.__user_state__[tg_user_id].event_data.payload.data.logs)

    def get_logs(
        self,
        tg_user_id: int
    ):
        if tg_user_id in self.__user_state__:
            return self.__user_state__[tg_user_id].event_data.payload.data.logs
        else:
            return []

    def set_logs(
        self,
        tg_user_id: int,
        logs: list[dict]
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.data.logs = logs
        except Exception as e:
            log.error(f"{{ user_state.set_logs }} -> {e}")

    def change_status(
        self,
        tg_user_id: int,
        status: AppStatusEnum,
        isBotRunning: AppStatusEnum
    ):
        try:
            if tg_user_id in self.__user_state__:
                self.__user_state__[tg_user_id].event_data.payload.data.status = status
                self.__user_state__[tg_user_id].event_data.payload.data.isBotRunning = isBotRunning
        except Exception as e:
            log.error(f"{{ user_state.change_status }} -> {e}")

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
                self.__user_state__[tg_user_id].event_data.payload.data.isSleeping = new_status
        except Exception as e:
            log.error(f"{{ user_state.change_sleeping_status }} -> {e}")