#ifndef CORE_SERVICE_H
#define CORE_SERVICE_H

#include <math.h>
#include <stdint.h>
#include <stdlib.h>

#define PI 3.141592653589793238462643383279502884f
#define True 1
#define False 0 // Falsity Phonk

#define SINE_FORMULA     sinf(((float)phase / 4294967296.0f) * 2.0f * PI)
#define TRIANGLE_FORMULA (1.0f - (float)((int32_t)(phase ^ (phase & 0x80000000 ? 0xFFFFFFFF : 0))) / 1073741824.0f)
#define SAWTOOTH_FORMULA (((float)phase / 2147483648.0f) - 1.0f)
#define SQUARE_FORMULA   (phase < (uint32_t)(pulse_width * 4294967295.0f) ? 1.0f : -1.0f)
#define NOISE_FORMULA    (((float)rand() / (float)RAND_MAX) * 2.0f - 1.0f)

#define PHASE_INCREMENT (uint32_t)(((double)frequency / (double)sample_rate) * 4294967296.0)

typedef enum Waveforms {
    Sine, Triangle, Sawtooth, Square, Noise
    // SineCubed, SineC, RampUp, RampDown, Step, Lorenz
    // WhiteN, PinkN, BrownN, BlueN, PurpleN, GreyN, VelvetN
} Waveforms;

typedef struct Oscillator {
    Waveforms waveform;

    uint32_t phase;
    uint32_t phase_increment;

    float frequency;
    float pulse_width;
    int detune;
    
    float coarse;
    float fine;
    
    float amplitude;
} Oscillator;

typedef enum BitDepth {SixteenBit, TwentyFourBit, ThirtyTwoFloat} BitDepth;
typedef enum BufferSize {RealTime = 64, VeryLow = 128, LowMedium = 256, Low = 512, Medium = 1024, High = 2048, VeryHigh = 4096, SafeMode = 8192} BufferSize;

Oscillator OscillatorDefault();

#endif
