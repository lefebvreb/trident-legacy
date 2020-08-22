__kernel void initialise(__global _Complex float *buffer, uint initial_state) {
    uint global_id = get_global_id(0);

    buffer[global_id] = (global_id == initial_state) ? 1 : 0;
}

__kernel void add(__global _Complex float* buffer, _Complex float scalar) {
    buffer[get_global_id(0)] += scalar;
}