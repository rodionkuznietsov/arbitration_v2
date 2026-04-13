import time
from typing import Optional

import structlog

from ..cache import push_to_subscribes
from ..schemas import EventDataTypeEnum, ExchangeClearPayload, ExchangeEventData, ExchangeEventEnum, ExchangePayload, ExchangeSchema, MessageData

log: structlog.PrintLogger = structlog.get_logger()

class NotifyMassager:
    def push_exchange_message(
        self,
        exchange_data: Optional[ExchangeSchema] = None,
        event: Optional[ExchangeEventEnum] = None
    ):        
        if exchange_data is not None:
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
            log.info(f"{{ notify_manager.push_exchange_message.{event} }} -> {exchange_data.name.lower()}")

        elif event == ExchangeEventEnum.ClearExchanges:
            message = MessageData(
                event_data=ExchangeEventData(
                    type=EventDataTypeEnum.Exchange,
                    payload=ExchangeClearPayload(
                        event=event
                    ),
                    timestamp=int(time.time())
                )
            )

            push_to_subscribes(message=message)

            log.info(f"{{ notify_manager.push_exchange_message.{event} }} -> успешно")
        
notify_manager = NotifyMassager()