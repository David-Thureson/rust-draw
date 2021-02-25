
#[derive(Clone, Debug)]
pub struct PointF64 {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl PointF64 {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
        }
    }

    pub fn gradiant_one(&self, other: &Self, step_count: usize, step_index: usize) -> Self {
        debug_assert!(step_index <= step_count);
        let step_count = step_count as f64;
        let step_index = step_index as f64;
        let x_step_size = (other.x - self.x) / step_count;
        let y_step_size = (other.y - self.y) / step_count;
        let x = self.x + (x_step_size * step_index);
        let y = self.y + (y_step_size * step_index);
        Self::new(x, y)
    }
}

/*
pub fn gradient_point64(from: PointF64, to: PointF64, step_count: usize) -> Vec<PointF64> {
    let x_values = gradient_f64(from.0, to.0, step_count);
    let y_values = gradient_f64(from.1, to.1, step_count);
    let v = x_values.iter()
        .zip(y_values.iter())
        .map(|(x, y)| (*x, *y))
        .collect::<Vec<_>>();
    debug_assert_eq!(step_count + 1, v.len());
    v
}
*/