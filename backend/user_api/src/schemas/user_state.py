class UserStateError(Exception):
    def __init__(self, status: int, msg: str):
        self.status = status
        self.msg = msg
        super().__init__(msg)