from typing import List, Tuple, Set
from pyray import Vector2

class Body:
    def __init__(
        self,
        entity_index: int,
        body_index: int,
        pos: Vector2,
        radius: float,
        is_static: bool,
    ): ...
    def collided(self, other: "Body") -> bool: ...
    def get_bounds(self) -> Tuple[float, float, float, float]: ...

class Collision:
    def __init__(
        self,
        self_entity_index: int,
        other_entity_index: int,
        self_body_index: int,
        other_body_index: int,
    ):
        self.self_entity_index: int
        self.other_entity_index: int
        self.self_body_index: int
        self.other_body_index: int

class GridPhysics:
    def __init__(self, size: int, cell_size: int):
        self.grid: List[List[Body]]
        self.dynamic_bodies: dict[int, List[Body]]
        self.size: int
        self.cell_size: int
        self.grid_size: int

    def reset(self) -> None: ...
    def get_grid_bounds(
        self, bounds: Tuple[float, float, float, float]
    ) -> Tuple[int, int, int, int]: ...
    def add_circle(
        self,
        entity_index: int,
        pos: Vector2,
        radius: float,
        body_index: int,
        is_static: bool,
    ) -> None: ...
    def add_static_circle(
        self, entity_index: int, pos: Vector2, radius: float
    ) -> None: ...
    def add_static_circles(
        self, entity_index: int, bodies: List[Vector2], radius: float
    ) -> None: ...
    def add_dynamic_circle(
        self, entity_index: int, pos: Vector2, radius: float
    ) -> None: ...
    def add_dynamic_circles(
        self, entity_index: int, bodies: List[Vector2], radius: float
    ) -> None: ...
    def get_collisions(self) -> Set[Collision]: ...
    def get_collisions_within_area(
        self, position: Vector2, radius: float
    ) -> Set[int]: ...

