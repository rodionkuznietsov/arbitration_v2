import time
import structlog

from ..schemas import EventTypeEnum, MessageContext, MessageData, MessageEventData, MessageMethod, UserStatePayload, UserStateError

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
        
            raise UserStateError(status=404, msg=f"Не удалось найти пользователя с id: {tg_user_id}")
        except UserStateError as e:
            log.error(e)
    