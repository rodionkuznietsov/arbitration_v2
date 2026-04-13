import json
import time
from typing import Optional

import structlog

from ..cache import push_to_subscribes
from ..schemas import AppStatusEnum, EventDataTypeEnum, EventTypeEnum, ExchangeClearPayload, ExchangeEventData, ExchangeEventEnum, ExchangePayload, ExchangeSchema, MessageContext, MessageData, MessageEventData, MessageEventPayload, MessageMethod
from ..core.state import user_state

log: structlog.PrintLogger = structlog.get_logger()

class NotifyMassager:
    def push_user_state_message(
        self
    ):
        pass

    def push_websocket_message(
        self,
        tg_user_id: int,
        data
    ):

        log.info(data)

        ws_message = MessageData(
            event_data=MessageEventData(
                type=EventDataTypeEnum.Websocket,
                payload=MessageEventPayload(
                    event=EventTypeEnum.Websocket,
                    symbol=user_state.long_active_symbol(tg_user_id),
                    longExchange=user_state.long_active_exchange(tg_user_id),
                    longOrderType=user_state.long_active_order_type(tg_user_id),
                    shortExchange=user_state.short_active_exchange(tg_user_id),
                    shortOrderType=user_state.short_active_order_type(tg_user_id),
                    status=AppStatusEnum.Online,
                    isBotRunning=True
                ),
                timestamp=int(time()),
                ws_data=data
            ),
            context=MessageContext(
                method=MessageMethod.WebsocketConnected,
                tg_user_id=tg_user_id
            )
        )

        push_to_subscribes(ws_message)

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