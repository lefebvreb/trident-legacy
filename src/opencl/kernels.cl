//#################################################################################################
//
//                                       Helper functions
// 
//#################################################################################################

// Multiply two float2 together as if they were complex numbers
static inline float2 complex_mul(
    const float2 lhs,
    const float2 rhs
) {
    return lhs.xx * rhs + lhs.yy * (float2) (-rhs.y, rhs.x);
}

// Helper function for the apply_gate* kernels
static inline size_t nth_cleared(
    size_t n,
    uchar target
) {
    size_t mask = ((size_t) 1 << target) - 1;
    return (n & mask) | ((n & ~mask) << 1);
}

// Returns the proper index corresponding to the #id element at the #pass level of the
// distribution vector
static inline size_t index(
    size_t pass,
    size_t id
) {
    return (1 << pass) * (1 + (id << 1)) - 1;
}

// Returns a prng float in the range [0,1] given a seed and the global id
static inline float random(
    const uint seed,
    const uint global_id
) {
    uint x = ~(seed * (global_id + 19));

    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    return (float) ((double) x * 2.3283064370807974e-10);
}

//#################################################################################################
//
//                                             Kernels
// 
//#################################################################################################

// Apply the gate [[u00, u01], [u10, u11]] to the #target qbit of the amplitudes buffer
kernel void apply_gate(
    global float2 *buffer,
    const uchar target,
    const float2 u00,
    const float2 u01,
    const float2 u10,
    const float2 u11
) {
    const size_t global_id = get_global_id(0);

    const size_t zero_state = nth_cleared(global_id, target);
    const size_t one_state  = zero_state | ((size_t) 1 << target);

    const float2 zero_amp = buffer[zero_state];
    const float2 one_amp  = buffer[one_state];

    buffer[zero_state] = complex_mul(u00, zero_amp) + complex_mul(u01, one_amp);
    buffer[one_state]  = complex_mul(u10, zero_amp) + complex_mul(u11, one_amp);
}

// Apply the gate [[u00, u01], [u10, u11]] to the #target qbit of the amplitudes buffer
// with qbit #control as control 
kernel void apply_controlled_gate(
    global float2 *buffer,
    const uchar target,
    const float2 u00,
    const float2 u01,
    const float2 u10,
    const float2 u11,
    const uchar control
) {
    const size_t global_id = get_global_id(0);

    const size_t zero_state = nth_cleared(global_id, target);
    const size_t one_state  = zero_state | ((size_t) 1 << target);

    const bool control_var_zero = (((size_t) 1 << control) & zero_state) > 0;
    const bool control_var_one  = (((size_t) 1 << control) & one_state) > 0;

    const float2 zero_amp = buffer[zero_state];
    const float2 one_amp  = buffer[one_state];   

    if (control_var_zero) {
        buffer[zero_state] = complex_mul(u00, zero_amp) + complex_mul(u01, one_amp);
    }

    if (control_var_one) {     
        buffer[one_state]  = complex_mul(u10, zero_amp) + complex_mul(u11, one_amp);
    }
}

// Calculate the probabilites by calculating the squared norm of all complex numbers in the buffer
// and storing the results in their real parts
kernel void calculate_probabilities(
    global float2 *buffer
) {
    const size_t global_id = get_global_id(0);

    float2 value = buffer[global_id];
    value *= value;    
    buffer[global_id].x = value.x + value.y;
}

// Reduce the distribution vector
kernel void reduce_distribution(
    global float *buffer,
    const uchar pass
) {
    const size_t global_id = get_global_id(0);

    const size_t id0 = index(pass-1, global_id << 1);
    const size_t id1 = index(pass-1, (global_id << 1) + 1);
    const size_t id  = index(pass, global_id);

    buffer[id] = buffer[id0] + buffer[id1];
}

// Perform measurements by traversing the distribution vector
kernel void do_measurements(
    global const float *buffer,
    global ulong *mesures,
    uchar size,
    const uint seed
) {
    const size_t global_id = get_global_id(0);
    const float rand = random(seed, global_id);

    size_t id = 0;
    float sum = 0.0;

    for (size--; size; size--) {
        const float value = buffer[index(size, id)];

        if (rand > sum + value) {
            sum += value;
            id++;
        }

        id <<= 1;
    }

    if (rand > sum + buffer[id << 1]) {
        id++;
    }

    mesures[global_id] = (ulong) id;
}