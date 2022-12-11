// | (c) 2022 Tremeschin, MIT License | ViyLine Project | //
use crate::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Default, Clone)]
pub struct Curve {
    pub points: Vec<Point>,

    // Curve parameters
    pub C: f64,
    pub A: f64,
    pub B: f64
}

impl Curve {
    // Returns the coefficients
    pub fn calculateCoefficients(&mut self) {

        // If we even have some points
        if self.points.len() > 0 {

            // Initial guess of B value
            if self.B == 0.0 {self.B = 0.5;}

            // This is one of the hardest part, find the perfect initial value I(0)
            let maxY = self.minMaxY().unwrap().1;

            // Repeat until we get a nice estimate of B
            for _ in 1..50 {

                // Update A coefficient based on last iteration values
                self.A = maxY + self.B;

                // X, Y points for linear regression
                let x = Vec::from_iter(self.points.iter().map( |point|  point.x                ));
                let y = Vec::from_iter(self.points.iter().map( |point| (self.A*(1.0) - point.y).ln() ));

                // Linear regression
                let sumX:   f64 = x.iter().sum();
                let sumY:   f64 = y.iter().sum();
                let sumXY:  f64 = x.iter().zip(y).map(|(a, b)| a*b).sum();
                let sumXSq: f64 = x.iter().map(|a| a*a).sum();
                let n:      f64 = self.points.len() as f64;

                // y = ax + b
                let a = (n*sumXY - sumX*sumY)/(n*sumXSq - sumX.powf(2.0));
                let b = (sumY - a*sumX)/n;

                // On the linearized iv curve, for y = A - Be^Cx, we have ln(y) = -Cx + ln(B)
                // So C = a and ln(B) = b
                self.C = a;
                self.B = exp(b);
            }
        }
    }

    // Calculate a generic point X
    pub fn interpolatedValueAt(&self, x: f64) -> f64 {
        return self.A - self.B*exp(self.C*x);
    }

    // Minimum and maximum value of the curve
    fn minMaxY(&self) -> Option<(f64, f64)> {
        if self.points.len() == 0 {return None;}
        let mut yValues = Vec::from_iter(self.points.iter().map(|point| point.y));
        yValues.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let minY = yValues.first().unwrap();
        let maxY = yValues.last().unwrap();
        return Some((*minY, *maxY));
    }

    // Empty the curve
    pub fn clear(&mut self) {
        self.points = Vec::new();
        self.A = 0.0;
        self.B = 0.0;
        self.C = 0.0;
    }

    pub fn addPoint(&mut self, x: f64, y: f64) {
        self.points.push(Point { x: x, y: y });
    }
}
