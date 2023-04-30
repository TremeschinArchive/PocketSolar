// Arduino UNO code for the PocketSolar project

// Pulse Width Modulation configuration
#define PWM_PIN 13
#define PWM_FREQUENCY 5000.0
#define PWM_PERIOD 1.0 / PWM_FREQUENCY
#define PWM_SECONDS_PER_MEASURE 0.2
#define DUTY_CYCLE_DELTA_ITERATION 0.05

// Measurements
#define MEASURE_VOLTAGE_PIN A0
#define MEASURE_CURRENT_PIN A1

float dutyCycle = 0.0;

void setup() {
    // Configure Input/Output pins
    pinMode(PWM_PIN, OUTPUT);
    pinMode(MEASURE_VOLTAGE_PIN, INPUT);
    pinMode(MEASURE_CURRENT_PIN, INPUT);

    // Start serial communication
    Serial.begin(9600);
}

// Main loop: perform a "PWM Sweep" varying the duty cycle on the Buck converter,
// read voltage and current, send it to the serial port for Rust to read
void loop() {
    dutyCycle += DUTY_CYCLE_DELTA_ITERATION;
    if (dutyCycle > 1.0) dutyCycle = 0.0;

    // Apply PWM until stable output current
    pwm(PWM_PIN, dutyCycle, PWM_SECONDS_PER_MEASURE);

    // Send values to Serial, shouldn't take much apart from dutyCycle float?
    Serial.print(analogRead(MEASURE_VOLTAGE_PIN));
    Serial.print(",");
    Serial.print(analogRead(MEASURE_CURRENT_PIN));
    Serial.print(",");
    Serial.println(dutyCycle, 3);
}

// An effort to make a better delayMicroseconds() function
// https://www.arduino.cc/reference/en/language/functions/time/delaymicroseconds/
// delayMicroseconds() "the largest value that will produce an accurate delay is 16383"
void betterDelayMicroseconds(unsigned long us) {
    unsigned long start = micros();
    while(micros() < start + us);
}

// Digital PWM on a pin, at a certain dutyCycle, for a certain duration
// NOTE: Fixed frequency for fast(er) code
// NOTE: duration = max(1/frequency, duration)
void pwm(int pin, float dutyCycle, float duration) {

    // Start time
    unsigned long start = millis();

    // Calculate on and off time based on duty cycle
    float onTime  = PWM_PERIOD * dutyCycle;
    float offTime = PWM_PERIOD * (1 - dutyCycle);

    // Loop until the time is up
    while(millis() < start + (unsigned long)(PWM_SECONDS_PER_MEASURE*1000.0)){
        digitalWrite(pin, HIGH);
        betterDelayMicroseconds(onTime * 1000000);
        digitalWrite(pin, LOW);
        betterDelayMicroseconds(offTime * 1000000);
    }
}


