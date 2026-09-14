#include <CoreService.h>

Oscillator OscillatorDefault() {
    uint32_t sample_rate = 44100;
    float frequency = 440.0f;
    
    return (Oscillator) {
        .waveform = Sine,
        .phase = 0,
        .phase_increment = (uint32_t)(((double)frequency / (double)sample_rate) * 4294967296.0),
        .frequency = frequency,
        .pulse_width = 0.5f,
        .detune = 0,
        .coarse = 0.0f,
        .fine = 0.0f,
        .amplitude = 1.0f
    };
}
