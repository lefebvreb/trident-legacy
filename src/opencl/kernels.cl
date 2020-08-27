inline static size_t nth_cleared(
    size_t n,
    uchar target
) {
    size_t mask = ((size_t) 1 << target) - 1;
    return (n & mask) | ((n & ~mask) << 1);
}

kernel void apply_gate(
    global _Complex float *amplitudes,
    const uchar target,
    const _Complex float u00,
    const _Complex float u01,
    const _Complex float u10,
    const _Complex float u11
) {
    const size_t global_id = get_global_id(0);

    const size_t zero_state = nth_cleared(global_id, target);
    const size_t one_state  = zero_state | ((size_t) 1 << target);

    const _Complex float zero_amp = amplitudes[zero_state];
    const _Complex float one_amp  = amplitudes[one_state];

    amplitudes[zero_state] = u00*zero_amp + u01*one_amp;
    amplitudes[one_state]  = u10*zero_amp + u11*one_amp;
}

kernel void apply_controlled_gate(
    global _Complex float *amplitudes,
    const uchar target,
    const _Complex float u00,
    const _Complex float u01,
    const _Complex float u10,
    const _Complex float u11,
    const uchar control
) {
    const size_t global_id = get_global_id(0);

    const size_t zero_state = nth_cleared(global_id, target);
    const size_t one_state  = zero_state | ((size_t) 1 << target);

    const bool control_var_zero = (((size_t) 1 << control) & zero_state) > 0;
    const bool control_var_one  = (((size_t) 1 << control) & one_state) > 0;

    const _Complex float zero_amp = amplitudes[zero_state];
    const _Complex float one_amp  = amplitudes[one_state];   

    if (control_var_zero) {
        amplitudes[zero_state] = u00*zero_amp + u01*one_amp;
    }

    if (control_var_one) {     
        amplitudes[one_state]  = u10*zero_amp + u11*one_amp;
    }
}

kernel void calculate_probabilities(
    global _Complex float *buffer
) {
    const size_t global_id = get_global_id(0);

    union {
    _Complex float c;
    float2 f;
    } v = {.c = buffer[global_id]};

    v.f *= v.f;
    v.f.x = v.f.x + v.f.y;
    
    buffer[global_id] = v.c;
}

inline size_t index(
    size_t n,
    size_t i
) {
    return (1 << n) * (1 + (i << 1)) - 1;
}

// Pass in [1, size)
kernel void reduce_distribution(
    global float *distribution,
    const uchar pass
) {
    const size_t global_id = get_global_id(0);

    const size_t id0 = index(pass-1, global_id << 1);
    const size_t id1 = index(pass-1, (global_id << 1) + 1);
    const size_t id  = index(pass, global_id);

    distribution[id] = distribution[id0] + distribution[id1];
}

// Modified 32-bit xorshift algorithm. Supposed to give uniform floats in the range [0-1]
inline float random(
    const uint seed,
    const uint global_id
) {
    uint x = (seed * (global_id + 19)) ^ 0xFFFFFFFF;

    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    return (float) ((double) x * 2.3283064370807974e-10);
}

kernel void do_measurements(
    const global float *distribution,
    global ulong *measurements,
    uchar size,
    const uint seed
) {
    const size_t global_id = get_global_id(0);
    const float value = random(seed, global_id);

    size_t state = 0;
    float sum = 0.0;

    for (size--; size; size--) {
        const float distrib = distribution[index(size, state)];

        if (value > sum + distrib) {
            sum += distrib;
            state++;
        }

        state <<= 1;
    }

    if (value > sum + distribution[state << 1]) {
        state++;
    }

    measurements[global_id] = (ulong) state;
}