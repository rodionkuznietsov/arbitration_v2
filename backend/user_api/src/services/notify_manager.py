from email import message
import json
import time
from typing import Optional

import structlog

from ..db import database

from ..cache import push_to_subscribes
from ..schemas import EventTypeEnum, ExchangeEnum, LogPayload, LogStatusEnum, OrderTypeEnum, UserStatePayload, WebsocketPayload, EventDataTypeEnum, ExchangeClearPayload, ExchangeEventData, ExchangeEventEnum, ExchangePayload, ExchangeSchema, MessageContext, MessageData, MessageEventData, MessageEventPayload, MessageMethod
from ..core.state import user_state

log: structlog.PrintLogger = structlog.get_logger()

class NotifyMassager:
    def push_user_state_message(
        self,
        tg_user_id: int
    ) -> None:
        push_to_subscribes(user_state.get(tg_user_id))

    async def push_log_message(
        self,
        tg_user_id: int,
        event: EventTypeEnum,
        
        symbol: str,
        longExchange: ExchangeEnum,
        shortExchange: ExchangeEnum,
        longOrderType: OrderTypeEnum,
        shortOrderType: OrderTypeEnum,

        status: LogStatusEnum,
        data
    ):
        message = MessageData(
            event_data=MessageEventData(
                type=EventDataTypeEnum.Log,
                timestamp=int(time.time()),
                payload=LogPayload(
                    event=event,

                    symbol=symbol,

                    longExchange=longExchange,
                    longOrderType=longOrderType,

                    shortExchange=shortExchange,                    
                    shortOrderType=shortOrderType,

                    status=status
                )
            ),
            context=MessageContext(
                method=MessageMethod.User,
                tg_user_id=tg_user_id
            )
        )

        push_to_subscribes(message=message)
        await database.add_log(tg_user_id, data=data)

    def push_websocket_message(
        self,
        tg_user_id: int,
        data
    ):

        try:
            ws_message = MessageData(
                event_data=MessageEventData(
                    type=EventDataTypeEnum.Websocket,
                    payload=WebsocketPayload(
                        channel=data.get("channel"),
                        result=data.get("result")
                    ),
                    timestamp=int(time.time()),
                ),
                context=MessageContext(
                    method=MessageMethod.WebsocketConnected,
                    tg_user_id=tg_user_id
                )
            )

            push_to_subscribes(ws_message)
        except Exception as e:
            log.error(f"{{ notify_manager.push_websocket_message }} -> {e}")

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