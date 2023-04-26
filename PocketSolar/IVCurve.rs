use crate::*;

BrokenStruct! {
    pub struct IVPoint {
        pub v: f64,
        pub i: f64,
    }
}

BrokenStruct! {
    pub struct IVCurve {
        pub points: Vec<IVPoint>,

        // IV Curve parameters
        pub C: f64,
        pub A: f64,
        pub B: f64,

        // MPP
        pub MPPVoltage: f64,
    }
}

impl PocketSolarApp {

    // Short hand for extra functionality
    pub fn updateSolarPanelCurve(&mut self) {
        if self.recalculateRegressionOnCoefficientChanges {
            self.solarPanelCurve.clearRegression();
        }

        // Calculate regression after measurement
        self.solarPanelCurve.calculateCoefficients(self.regressionSteps);
        self.solarPanelCurve.calculateMPP();
    }
}

impl IVCurve {

    // Returns the coefficients
    pub fn calculateCoefficients(&mut self, steps: i64) {

        // If we even have some points
        if self.points.len() > 0 {

            // Initial guess of B value
            if self.B == 0.0 {self.B = 0.5;}

            // This is one of the hardest part, find the perfect initial value I(0)
            let maxY = self.minMaxY().unwrap().1;

            // Repeat until we get a nice estimate of B
            for _ in 1..=steps {
                // Update A coefficient based on last iteration values
                self.A = maxY + self.B;

                // X, Y points for linear regression
                let x = Vec::from_iter(self.points.iter().map( |point|  point.v                ));
                let y = Vec::from_iter(self.points.iter().map( |point| (self.A*(1.0) - point.i).ln() ));

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

    // Calculate the maximum power point by matching when the derivative of the power curve is 0
    pub fn calculateMPP(&mut self) {
        self.MPPVoltage = 0.0;
        let delta = 0.01;

        while self.powerAtVoltage(self.MPPVoltage + delta) > self.powerAtVoltage(self.MPPVoltage) {
            self.MPPVoltage += delta;
        }
    }

    // Calculate the power at a given voltage
    pub fn powerAtVoltage(&self, voltage: f64) -> f64 {
        return voltage*self.currentAtVoltage(voltage);
    }

    // Calculate the current at a given voltage
    pub fn currentAtVoltage(&self, voltage: f64) -> f64 {
        return self.A - self.B*exp(self.C*voltage);
    }

    // Minimum and maximum value of the curve
    fn minMaxY(&self) -> Option<(f64, f64)> {
        if self.points.len() == 0 {return None;}
        let mut yValues = Vec::from_iter(self.points.iter().map(|point| point.i));
        yValues.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let minY = yValues.first().unwrap();
        let maxY = yValues.last().unwrap();
        return Some((*minY, *maxY));
    }

    pub fn clearRegression(&mut self) {
        self.A = 0.0;
        self.B = 0.0;
        self.C = 0.0;
    }

    // Empty the curve
    pub fn clear(&mut self) {
        self.points = Vec::new();
        self.clearRegression();
    }

    pub fn addPoint(&mut self, x: f64, y: f64) {
        self.points.push(IVPoint { v: x, i: y });
    }
}
