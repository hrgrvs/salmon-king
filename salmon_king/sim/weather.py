from __future__ import annotations

from salmon_king.data.camps import CampDef
from salmon_king.sim.clock import doy
from salmon_king.sim.models import Weather
from salmon_king.sim.rng import Rng


def roll_weather(day, camp: CampDef, rng: Rng, force: Weather | None = None) -> Weather:
    if force:
        return force
    n = doy(day)
    # June–July advection fog; September gets mean
    fog_p = camp.fog + (0.18 if 152 <= n <= 212 else 0.0)
    will_p = camp.williwaw + (0.08 if n >= 244 else 0.0)
    wind = max(4.0, rng.gauss(14.0, 6.0))
    if n >= 244:
        wind += rng.uniform(2, 10)
    seas = max(0.5, wind / 6.0 + rng.uniform(-0.4, 1.2))
    if camp.district == "central":
        seas += 0.4  # Shelikof short steep
    fog = min(1.0, max(0.0, rng.gauss(fog_p, 0.15)))
    williwaw = rng.random() < will_p * 0.08
    if williwaw:
        wind = max(wind, rng.uniform(42, 68))
        seas = max(seas, rng.uniform(4.5, 8.0))
        precip = "williwaw"
        label = f"WILLIWAW {int(wind)} kt, {seas:.0f} ft"
    elif fog > 0.55:
        precip = "fog"
        label = f"advection fog, {int(wind)} kt"
    elif rng.random() < 0.45:
        precip = "rain"
        label = f"rain {int(wind)} kt, {seas:.0f} ft"
    else:
        precip = "overcast"
        label = f"overcast {int(wind)} kt, {seas:.0f} ft"
    return Weather(
        wind_kt=round(wind, 1),
        seas_ft=round(seas, 1),
        fog=round(fog, 2),
        precip=precip,
        williwaw=williwaw,
        label=label,
    )


def skiffs_grounded(weather: Weather) -> bool:
    return weather.williwaw or weather.seas_ft >= 7.5 or weather.wind_kt >= 45
