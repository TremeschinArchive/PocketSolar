// Arduino UNO code for the ViyLine project

#define PWM_PIN 13
#define PWM_FREQUENCY 100.0
#define PWM_SECONDS_PER_MEASURE 0.2
#define PWM_POINTS_PER_MEASURE 100

void setup() {
    // Set PWM pin as output
    pinMode(PWM_PIN, OUTPUT);

    // Start serial communication
    Serial.begin(9600);
}

// https://www.arduino.cc/reference/en/language/functions/time/delaymicroseconds/
// delayMicroseconds() "the largest value that will produce an accurate delay is 16383"
//
// An effort to make a better delayMicroseconds() function
void betterDelayMicroseconds(unsigned long us) {
    unsigned long start = micros();
    while(micros() < start + us);
}

// Digital PWM on a pin, at a certain frequency and dutyCycle, for a certain duration
// NOTE: duration = max(1/frequency, duration)
void pwm(int pin, float frequency, float dutyCycle, float duration) {

    // Start time
    unsigned long start = millis();

    // Period in seconds of a single cycle
    float period = 1.0 / frequency;

    // Calculate on and off time based on duty cycle
    float onTime  = period * dutyCycle;
    float offTime = period * (1 - dutyCycle);

    // Loop until the time is up
    while(millis() < start + (unsigned long)(duration*1000.0)){
        digitalWrite(pin, HIGH);
        betterDelayMicroseconds(onTime * 1000000);
        digitalWrite(pin, LOW);
        betterDelayMicroseconds(offTime * 1000000);
    }
}

// Main loop: perform a "PWM Sweep" varying the duty cycle, read voltage
// and current, send it to the serial port
void loop() {
    for (float dutyCycle=0.0; dutyCycle<1.0; dutyCycle+=(1.0/PWM_POINTS_PER_MEASURE)) {
        pwm(PWM_PIN, PWM_FREQUENCY, dutyCycle, 0.1);
        // Serial.println(dutyCycle);
    }
}
