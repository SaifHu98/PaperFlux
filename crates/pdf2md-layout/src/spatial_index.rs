use pdf2md_ast::geometry::{BoundingBox, Point, Rect};

#[derive(Debug, Clone)]
pub struct SpatialItem<T: Clone> {
    pub id: usize,
    pub bbox: BoundingBox,
    pub data: T,
}

pub struct SpatialIndex2D<T: Clone> {
    items: Vec<SpatialItem<T>>,
}

impl<T: Clone> Default for SpatialIndex2D<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T: Clone> SpatialIndex2D<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, bbox: BoundingBox, data: T) -> usize {
        let id = self.items.len();
        self.items.push(SpatialItem { id, bbox, data });
        id
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SpatialItem<T>> {
        self.items.iter()
    }

    /// Finds all items whose bounding box intersects the given query rectangle.
    pub fn query_intersecting(&self, query: &Rect) -> Vec<&SpatialItem<T>> {
        self.items
            .iter()
            .filter(|item| item.bbox.intersects(query))
            .collect()
    }

    /// Finds all items contained entirely within the given query rectangle.
    pub fn query_contained(&self, query: &Rect) -> Vec<&SpatialItem<T>> {
        self.items
            .iter()
            .filter(|item| query.contains_rect(&item.bbox))
            .collect()
    }

    /// Finds the nearest item to a specific point.
    pub fn nearest_to_point(&self, point: Point) -> Option<(&SpatialItem<T>, f32)> {
        let mut best_item = None;
        let mut min_dist = f32::MAX;

        for item in &self.items {
            let dist = point.distance_to(&item.bbox.center());
            if dist < min_dist {
                min_dist = dist;
                best_item = Some(item);
            }
        }

        best_item.map(|item| (item, min_dist))
    }

    /// Finds the nearest item directly below a given bounding box (e.g. caption below a figure).
    pub fn nearest_below(&self, query: &Rect, max_distance: f32) -> Option<&SpatialItem<T>> {
        let mut best_item = None;
        let mut min_dy = max_distance;

        for item in &self.items {
            if item.bbox.y_min() >= query.y_max() {
                let dy = item.bbox.y_min() - query.y_max();
                let horizontal_overlap = item.bbox.horizontal_overlap(query);
                if dy <= min_dy && horizontal_overlap > 0.0 {
                    min_dy = dy;
                    best_item = Some(item);
                }
            }
        }

        best_item
    }

    /// Finds the nearest item directly above a given bounding box (e.g. caption above a table).
    pub fn nearest_above(&self, query: &Rect, max_distance: f32) -> Option<&SpatialItem<T>> {
        let mut best_item = None;
        let mut min_dy = max_distance;

        for item in &self.items {
            if item.bbox.y_max() <= query.y_min() {
                let dy = query.y_min() - item.bbox.y_max();
                let horizontal_overlap = item.bbox.horizontal_overlap(query);
                if dy <= min_dy && horizontal_overlap > 0.0 {
                    min_dy = dy;
                    best_item = Some(item);
                }
            }
        }

        best_item
    }
}
