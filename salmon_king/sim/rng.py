from __future__ import annotations

import random


class Rng:
    """Thin wrapper so the sim never touches the global RNG."""

    def __init__(self, seed: int) -> None:
        self.seed = int(seed)
        self._r = random.Random(self.seed)

    def random(self) -> float:
        return self._r.random()

    def uniform(self, a: float, b: float) -> float:
        return self._r.uniform(a, b)

    def randint(self, a: int, b: int) -> int:
        return self._r.randint(a, b)

    def choice(self, seq):
        return self._r.choice(seq)

    def gauss(self, mu: float, sigma: float) -> float:
        return self._r.gauss(mu, sigma)

    def weighted(self, items: list[tuple[object, float]]):
        total = sum(w for _, w in items)
        pick = self._r.random() * total
        acc = 0.0
        for item, w in items:
            acc += w
            if pick <= acc:
                return item
        return items[-1][0]
