import time

import structlog

from ..cache import push_to_subscribes
from ..schemas import EventDataTypeEnum, ExchangeEventData, ExchangeEventEnum, ExchangePayload, ExchangeSchema, MessageData

log: structlog.PrintLogger = structlog.get_logger()

class NotifyMassager:
    def push_exchange_message(
        self,
        exchange_data: ExchangeSchema,
        event: ExchangeEventEnum
    ):        
        message = MessageData(
            event_data=ExchangeEventData(
                type=EventDataTypeEnum.Exchange,
                payload=ExchangePayload(
                    event=event,
                    exchange_name=exchange_data.name.lower(),
                    is_available=exchange_data.is_available
                ),
                timestamp=int(time.time())
            )
        )

        push_to_subscribes(message=message)
        log.info(f"{{ notify_manager.push_exchange_message }} -> {exchange_data.name.lower()}")

notify_manager = NotifyMassager()