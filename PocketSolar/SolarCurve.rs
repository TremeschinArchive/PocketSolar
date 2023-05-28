use crate::*;

BrokenStruct! {
    #[derive(Clone)]
    pub struct Measurement {
        pub voltage: f64,
        pub current: f64,
        pub dutyCycle: f64,
    }
}

BrokenStruct! {
    #[derive(Clone)]
    pub struct SolarCurve {
        // Serial port
        pub portName: String,

        // Measurements
        #[default(100)]
        pub maxPoints: usize,
        pub points: Vec<Measurement>,

        // Current, voltage amplification factor
        #[default(1.0)]
        pub Ki: f64,
        #[default(1.0)]
        pub Kv: f64,

        // IV Curve regression parameters
        // Regression parameters
        pub C: f64,
        pub A: f64,
        #[default(1.0)]
        pub B: f64,

        // MPP
        pub MPPVoltage: f64,
    }
}

impl SpinWhole for SolarCurve {
    fn main(this: Arc<RwLock<Self>>) {
        loop {
            // Don't spam connections
            Thread::sleep(Duration::from_millis(100));

            // Store the state of the port we are connecting to detect if it changes
            let currentPort = this.read().unwrap().portName.clone();

            if let Ok(mut port) = serialport::new(currentPort.clone(), 9600).open() {
                // Fix unecessary Windows read blockage
                port.set_flow_control(serialport::FlowControl::Hardware).unwrap();
                port.write_data_terminal_ready(true).unwrap();

                let mut reader = BufReader::new(port);
                let mut lastMessageTimestamp = Instant::now();

                loop {
                    // Checks if portName has changed externally
                    if this.read().unwrap().portName != currentPort {break}

                    // No data received for a while, assume broken connection
                    if lastMessageTimestamp + Duration::from_secs(2) < Instant::now() {
                        info!("No Arduino message received recently, reconnecting");
                        break;
                    }

                    // Read next line if any
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() {
                        Thread::sleep(Duration::from_millis(100));
                        continue;
                    }

                    // Update last message time
                    lastMessageTimestamp = Instant::now();

                    // Arduino Line is "voltage,current,dutyCycle" (int/1023, int/1023, float)
                    let parts: Vec<f64> = line.trim().split(',').map(|x| x.parse().unwrap_or(0.0)).collect();

                    // Wrong length (incomplete message?)
                    if parts.len() != 3 {continue}

                    // Ignore if voltage and current are too low (noise, low duty?)
                    if parts[0] + parts[1] < 0.2 {continue}

                    // // Add new measurement
                    let mut this = this.write().unwrap();

                    // Create new measurement struct and append it
                    let measurement = Measurement::default()
                        .voltage(parts[0] * (5.0/1023.0) * this.Kv)
                        .current(parts[1] * (5.0/1023.0) * this.Ki)
                        .dutyCycle(parts[2]);

                    this.points.push(measurement);

                    // Remove old measurements (we push on the end, delete on start)
                    while this.points.len() > this.maxPoints {
                        this.points.remove(0);
                    }

                    // Update coefficients
                    this.update();
                }
            }
        }
    }
}

impl SolarCurve {
    pub fn update(&mut self) {
        if self.A.is_nan() || self.B.is_nan() || self.C.is_nan() {
            self.clearRegression();
        }
        self.calculateCoefficients();
        self.calculateMPP();
    }

    // Calculates next step and / or initial coefficients
    pub fn calculateCoefficients(&mut self) {

        // If we even have some points
        if self.points.len() > 0 {
            let maxVoltage = self.minMaxX().unwrap()[1];

            // This is one of the hardest part, find the perfect initial value I(0)
            // Get the average of all points up to X% nominal voltage
            let pointsBelowPercent = self.points.iter().filter(|point| point.voltage < maxVoltage*0.2).map(|point| point.current).collect::<Vec<f64>>();
            let maxY = pointsBelowPercent.iter().sum::<f64>() / pointsBelowPercent.len() as f64;

            // Repeat until we get a nice estimate of B
            for _ in 1..=4 {
                // We are overshooting the estimate, decrease B as it found a local optima
                if self.currentAtVoltage(0.0) > maxY*1.05 {
                    self.B *= 0.8;
                }

                // Update A coefficient based on last iteration values
                self.A = maxY + self.B;

                // X, Y points for linear regression
                let x = Vec::from_iter(self.points.iter().map(|point| point.voltage));
                let y = Vec::from_iter(self.points.iter().map(|point| (self.A - point.current).abs().ln()));

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
                self.B = exp(b) * 0.40;

                // The point where the
                let maxVoltageAnalytic = (self.A.ln() - self.B.ln())/self.C;
                self.C *= (maxVoltageAnalytic/maxVoltage) * 0.96;
            }
        }
    }

    // Calculate the maximum power point by matching when the derivative of the power curve is 0
    pub fn calculateMPP(&mut self) {
        self.MPPVoltage = 0.0;
        let delta = 0.01;

        while self.powerAtVoltage(self.MPPVoltage + delta) > self.powerAtVoltage(self.MPPVoltage) {
            self.MPPVoltage += delta;

            // Don't dead-lock on bad regression
            if self.MPPVoltage > 1000.0 {break;}
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

    pub fn MPPPower(&self) -> f64 {
        return self.powerAtVoltage(self.MPPVoltage);
    }

    // Minimum and maximum Y value of the curve
    pub fn minMaxY(&self) -> Option<[f64; 2]> {
        if self.points.len() == 0 {return None;}
        let (mut min, mut max): (f64, f64) = (0.0, 0.0);
        for point in self.points.iter() {
            max = max.max(point.current);
            min = min.min(point.current);
        }
        return Some([min, max]);
    }

    // Minimum and maximum X value of the curve
    pub fn minMaxX(&self) -> Option<[f64; 2]> {
        if self.points.len() == 0 {return None;}
        let (mut min, mut max): (f64, f64) = (0.0, 0.0);
        for point in self.points.iter() {
            max = max.max(point.voltage);
            min = min.min(point.voltage);
        }
        return Some([min, max]);
    }

    pub fn clearRegression(&mut self) {
        self.A = 0.0;
        self.B = 1.0;
        self.C = 0.0;
    }

    // Empty the curve
    pub fn clear(&mut self) {
        self.points = Vec::new();
        self.clearRegression();
    }
}
