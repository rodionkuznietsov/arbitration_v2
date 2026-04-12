from fastapi import HTTPException
import jwt
from jwt.exceptions import InvalidSubjectError
import structlog

from ..jwt_func import ALGORITHM, JWT_SECRET_KEY

log: structlog.PrintLogger = structlog.get_logger()

def authothicate(token):
    credentials_exception = HTTPException(
        status_code=401,
        detail="Не удалось проверить учетные данные",
        headers={"WWW-Authenticate": "Bearer"}
    )

    try:
        payload = jwt.decode(token, JWT_SECRET_KEY, algorithms=[ALGORITHM])
    except InvalidSubjectError as e:
        log.error(f"JWT TYPE ERROR: {e}")
        raise credentials_exception
    except jwt.InvalidTokenError as e:
        log.error(f"JWT TYPE ERROR: {e}")
        raise credentials_exception

    tg_user_id = payload.get('sub')
    if tg_user_id is None:
        raise credentials_exception
    
    return tg_user_id