from salmon_king.sim.engine import new_game


def make_game(**kwargs):
    kw = {"seed": 7, "camp_id": "uganik", "year": 2025}
    kw.update(kwargs)
    return new_game(**kw)
