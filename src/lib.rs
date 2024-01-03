use pyo3::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    hash::{self, Hash},
};

#[derive(Clone, Copy, FromPyObject)]
pub struct Vector2 {
    x: f32,
    y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct Body {
    entity_index: usize,
    body_index: usize,
    pos: Vector2,
    radius: f32,
    is_static: bool,
}

impl Body {
    fn new(
        entity_index: usize,
        body_index: usize,
        pos: Vector2,
        radius: f32,
        is_static: bool,
    ) -> Self {
        Self {
            entity_index,
            body_index,
            pos,
            radius,
            is_static,
        }
    }

    fn collided(&self, other: &Body) -> bool {
        // Static bodies don't collide with anything
        if self.is_static {
            return false;
        }

        // Don't collide with self
        if self.entity_index == other.entity_index {
            return false;
        }

        let distance = (self.pos.x - other.pos.x).powi(2) + (self.pos.y - other.pos.y).powi(2);
        let radius = (self.radius + other.radius).powi(2);
        distance <= radius
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        (
            self.pos.x - self.radius,
            self.pos.x + self.radius,
            self.pos.y - self.radius,
            self.pos.y + self.radius,
        )
    }
}

#[pyclass(get_all)]
pub struct Collision {
    self_entity_index: usize,
    other_entity_index: usize,
    self_body_index: usize,
    other_body_index: usize,
}

impl hash::Hash for Collision {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.self_entity_index.hash(state);
        self.other_entity_index.hash(state);
        self.self_body_index.hash(state);
        self.other_body_index.hash(state);
    }
}

impl PartialEq for Collision {
    fn eq(&self, other: &Self) -> bool {
        self.self_entity_index == other.self_entity_index
            && self.other_entity_index == other.other_entity_index
            && self.self_body_index == other.self_body_index
            && self.other_body_index == other.other_body_index
    }
}

impl Eq for Collision {}

#[pymethods]
impl Collision {
    #[new]
    pub fn new(
        self_entity_index: usize,
        other_entity_index: usize,
        self_body_index: usize,
        other_body_index: usize,
    ) -> Self {
        Self {
            self_entity_index,
            other_entity_index,
            self_body_index,
            other_body_index,
        }
    }
}

#[pyclass(module = "radyx")]
pub struct GridPhysics {
    grid: Vec<Vec<Body>>,
    dynamic_bodies: HashMap<usize, Vec<Body>>,
    size: usize,
    cell_size: usize,
    grid_size: usize,
}

#[pymethods]
impl GridPhysics {
    #[new]
    pub fn new(size: usize, cell_size: usize) -> Self {
        let grid_size = size / cell_size;
        let mut grid = Vec::with_capacity(grid_size * grid_size);
        for _ in 0..grid_size * grid_size {
            grid.push(Vec::new());
        }

        Self {
            grid,
            dynamic_bodies: HashMap::new(),
            size,
            cell_size,
            grid_size,
        }
    }

    pub fn reset(&mut self) {
        self.dynamic_bodies.clear();
        for cell in self.grid.iter_mut() {
            cell.clear();
        }
    }

    pub fn get_grid_bounds(&self, bounds: (f32, f32, f32, f32)) -> (usize, usize, usize, usize) {
        (
            (bounds.0 / (self.cell_size as f32)).floor() as usize,
            (bounds.1 / (self.cell_size as f32)).ceil() as usize,
            (bounds.2 / (self.cell_size as f32)).floor() as usize,
            (bounds.3 / (self.cell_size as f32)).ceil() as usize,
        )
    }

    pub fn add_circle(
        &mut self,
        entity_index: usize,
        pos: Vector2,
        radius: f32,
        body_index: usize,
        is_static: bool,
    ) {
        let body = Body::new(entity_index, body_index, pos, radius, is_static);

        let (lower_x, upper_x, lower_y, upper_y) = self.get_grid_bounds(body.get_bounds());

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                let cell = self.grid.get_mut(x * self.grid_size + y);
                if let Some(cell) = cell {
                    cell.push(body);
                }
            }
        }
    }

    pub fn add_static_circle(&mut self, entity_index: usize, pos: Vector2, radius: f32) {
        self.add_circle(entity_index, pos, radius, 0, true)
    }

    pub fn add_static_circles(&mut self, entity_index: usize, bodies: Vec<Vector2>, radius: f32) {
        for (i, pos) in bodies.iter().enumerate() {
            self.add_circle(entity_index, *pos, radius, i, true);
        }
    }

    pub fn add_dynamic_circle(&mut self, entity_index: usize, pos: Vector2, radius: f32) {
        self.add_circle(entity_index, pos, radius, 0, false);
        self.dynamic_bodies
            .entry(entity_index)
            .or_insert_with(Vec::new)
            .push(Body::new(entity_index, 0, pos, radius, false));
    }

    pub fn add_dynamic_circles(&mut self, entity_index: usize, bodies: Vec<Vector2>, radius: f32) {
        for (i, pos) in bodies.iter().enumerate() {
            self.add_circle(entity_index, *pos, radius, i, false);
            self.dynamic_bodies
                .entry(entity_index)
                .or_insert_with(Vec::new)
                .push(Body::new(entity_index, i, *pos, radius, false));
        }
    }

    pub fn get_collisions(&self) -> HashSet<Collision> {
        let mut collisions = HashSet::new();

        for (entity_index, bodies) in self.dynamic_bodies.iter() {
            for body in bodies.iter() {
                let (lower_x, upper_x, lower_y, upper_y) = self.get_grid_bounds(body.get_bounds());
                for x in lower_x..=upper_x {
                    for y in lower_y..=upper_y {
                        let cell = self.grid.get(x * self.grid_size + y);
                        if let Some(cell) = cell {
                            for other in cell.iter() {
                                if body.collided(other) {
                                    let collision = Collision::new(
                                        *entity_index,
                                        other.entity_index,
                                        body.body_index,
                                        other.body_index,
                                    );
                                    collisions.insert(collision);
                                }
                            }
                        }
                    }
                }
            }
        }
        collisions
    }

    pub fn get_collisions_within_area(&self, position: Vector2, radius: f32) -> HashSet<usize> {
        let (lower_x, upper_x, lower_y, upper_y) = self.get_grid_bounds((
            position.x - radius,
            position.x + radius,
            position.y - radius,
            position.y + radius,
        ));

        let mut collisions = HashSet::new();

        for x in lower_x..=upper_x {
            for y in lower_y..=upper_y {
                let cell = self.grid.get(x * self.grid_size + y);
                if let Some(cell) = cell {
                    for other in cell.iter() {
                        collisions.insert(other.entity_index);
                    }
                }
            }
        }
        collisions
    }
}

#[pymodule]
fn radyx(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Collision>()?;
    m.add_class::<GridPhysics>()?;
    m.add("__doc__", "Made in Rust!")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_dynamic_collisions() {
        let mut grid = GridPhysics::new(100, 10);
        grid.add_dynamic_circles(
            0,
            vec![
                Vector2::new(5.0, 5.0),
                Vector2::new(5.0, 5.5),
                Vector2::new(5.0, 6.0),
                Vector2::new(5.0, 6.5),
            ],
            1.0,
        );
        grid.add_dynamic_circles(
            1,
            vec![
                Vector2::new(5.0, 8.5),
                Vector2::new(5.0, 9.0),
                Vector2::new(5.0, 9.5),
                Vector2::new(5.0, 10.0),
            ],
            1.0,
        );

        let collisions = grid.get_collisions();
        assert_eq!(collisions.len(), 2);
        for collision in collisions.iter() {
            println!(
                "Self: {}, Other: {}, Self Body: {}, Other Body: {}",
                collision.self_entity_index,
                collision.other_entity_index,
                collision.self_body_index,
                collision.other_body_index
            );
        }
        assert!(collisions.contains(&Collision::new(1, 0, 0, 3)));
        assert!(collisions.contains(&Collision::new(0, 1, 3, 0)));
    }
}
