use crate::*;

pub type ShapeList = Vec<Shape>;

#[derive(Clone, Debug)]
pub enum Shape {
    Circle {
        //center: euclid::Point2D<f64, ScreenSpace>,
        center: PointF64,
        radius: f64,
        color: Color1,
    },
    Rectangle {
        top_left: PointF64,
        bottom_right: PointF64,
        color: Color1,
    },
    RectangleFast {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color_index: usize,
    }
}

impl Shape {

    pub fn circle(center: PointF64, radius: f64, color: Color1) -> Shape {
        Shape::Circle {
            center,
            radius,
            color,
        }
    }

    pub fn circle_xy(center_x: f64, center_y: f64, radius: f64, color: Color1) -> Shape {
        Shape::circle(PointF64::new(center_x, center_y), radius, color)
    }

    pub fn rectangle(top_left: PointF64, bottom_right: PointF64, color: Color1) -> Shape {
        Shape::Rectangle {
            top_left,
            bottom_right,
            color,
        }
    }

    pub fn rectangle_fast(x: f64, y: f64, width: f64, height: f64, color_index: usize) -> Shape {
        Shape::RectangleFast {
            x,
            y,
            width,
            height,
            color_index,
        }
    }

    pub fn gradiant_one(&self, other: &Self, step_count: usize, step_index: usize) -> Self {
        debug_assert!(step_index <= step_count);
        match (self, other) {
            (Self::Circle { center: from_center, radius: from_radius, color: from_color},
                    Self::Circle { center: to_center, radius: to_radius, color: to_color}) => {
                let center = from_center.gradiant_one(&to_center, step_count, step_index);
                let radius = gradiant_f64_one(*from_radius, *to_radius, step_count, step_index);
                let color = from_color.gradiant_one(to_color, step_count, step_index);
                Self::circle(center, radius, color)
            },
            _ => unimplemented!(),
        }
    }

    pub fn with_center(&self, new_center: PointF64) -> Self {
        match self {
            Self::Circle { center: _, radius, color} => Self::circle(new_center, *radius, color.clone()),
            _ => unimplemented!(),
        }
    }

    pub fn with_center_xy(&self, new_center_x: f64, new_center_y: f64) -> Self {
        self.with_center(PointF64::new(new_center_x, new_center_y))
    }

    pub fn with_radius(&self, new_radius: f64) -> Self {
        match self {
            Self::Circle { center, radius: _, color} => Self::circle(center.clone(), new_radius, color.clone()),
            _ => unimplemented!(),
        }
    }

    pub fn with_color(&self, new_color: Color1) -> Self {
        match self {
            Self::Circle { center, radius, color: _ } => Self::circle(center.clone(), *radius, new_color),
            Self::Rectangle { top_left, bottom_right, color: _ } => Self::rectangle(*top_left, *bottom_right, new_color),
            Self::RectangleFast { .. } => panic!("Not implemented for RectangleFast since it doesn't have a Color1."),
        }
    }
}

