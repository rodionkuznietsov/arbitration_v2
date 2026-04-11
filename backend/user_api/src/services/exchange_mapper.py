def exchange_mapper(
    raw_exchanges: list
):
    mapped = {}

    for raw_exchange in raw_exchanges:
        mapped[raw_exchange["name"]] = raw_exchange["is_available"]

    return mapped