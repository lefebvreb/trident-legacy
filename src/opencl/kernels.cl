static size_t nth_cleared(
    size_t n, 
    uchar target
) {
    size_t mask = ((size_t) 1 << target) - 1;

    return (n & mask) | ((n & ~mask) << 1);
}

__kernel void initialize(
    __global _Complex float *amplitudes, 
    ulong initial_state
) {
    size_t const global_id = get_global_id(0);

    amplitudes[global_id] = (global_id == initial_state) ? (float) 1 : (float) 0;
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

/*__kernel void measure(
    __global _Complex float *amplitudes,

) {

}*/