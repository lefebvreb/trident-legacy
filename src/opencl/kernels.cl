static size_t nth_cleared(
    size_t n, 
    uchar target
) {
    size_t mask = ((size_t) 1 << target) - 1;
    return (n & mask) | ((n & ~mask) << 1);
}

__kernel void apply_gate(
    __global _Complex float *amplitudes,
    const uchar target,
    const _Complex float u00,
    const _Complex float u01,
    const _Complex float u10,
    const _Complex float u11
) {
    const size_t gloabal_id = get_global_id(0);

    const size_t zero_state = nth_cleared(gloabal_id, target);
    const size_t one_state  = zero_state | ((size_t) 1 << target);

    const _Complex float zero_amp = amplitudes[zero_state];
    const _Complex float one_amp  = amplitudes[one_state];

    amplitudes[zero_state] = u00*zero_amp + u01*one_amp;
    amplitudes[one_state]  = u10*zero_amp + u11*one_amp;
}

__kernel void apply_controlled_gate(
    __global _Complex float *amplitudes,
    const uchar target,
    const _Complex float u00,
    const _Complex float u01,
    const _Complex float u10,
    const _Complex float u11,
    const uchar control
) {
    const size_t gloabal_id = get_global_id(0);

    const size_t zero_state = nth_cleared(gloabal_id, target);
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

__kernel void calculate_probabilities(
    const __global _Complex float *amplitudes,
    __global float *probabilities
) {
    const size_t global_id = get_global_id(0);

    union {
        float2 f;
        _Complex float c;
    } v = {.c = amplitudes[global_id]};

    v.f = v.f * v.f;

    probabilities[global_id] = v.f.x + v.f.y;
}

__kernel void calculate_distribution(
    const __global float *probabilities,
    __global float *distribution,
    const uchar size,
    const uchar pass
) {

}