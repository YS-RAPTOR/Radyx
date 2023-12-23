use pyo3::{exceptions::PyTypeError, prelude::*, types::PyDict};
use std::collections::HashMap;

#[derive(Clone, Copy, FromPyObject)]
pub struct Vector2 {
    x: f32,
    y: f32,
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

        for x in (lower_x..=upper_x).step_by(self.cell_size) {
            for y in (lower_y..=upper_y).step_by(self.cell_size) {
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
                .push(Body::new(entity_index, 0, *pos, radius, false));
        }
    }

    pub fn get_collisions(&self) -> Vec<Collision> {
        let mut collisions = Vec::new();

        for (entity_index, bodies) in self.dynamic_bodies.iter() {
            for body in bodies.iter() {
                let (lower_x, upper_x, lower_y, upper_y) = self.get_grid_bounds(body.get_bounds());
                for x in (lower_x..=upper_x).step_by(self.cell_size) {
                    for y in (lower_y..=upper_y).step_by(self.cell_size) {
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
                                    collisions.push(collision);
                                }
                            }
                        }
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
