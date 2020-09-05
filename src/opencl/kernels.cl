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

// Helper function for the apply_gate kernels
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

//#################################################################################################
//
//                                          MWC64X prng
// 
//#################################################################################################

// Returns (a+b) % m if a < m && b < m
static inline ulong modular_add64(ulong a, ulong b, ulong m) {
    ulong res = a + b;

    if ((res >= m) || (res < a)) res -= m;

    return res;
}

// Returns (a*b) % m if a < m && b < m
static inline ulong modular_mul64(ulong a, ulong b, ulong m) {
    ulong res = 0;

    while (a) {
        if (a & 1) res = modular_add64(res, b, m);
        b = modular_add64(b, b, m);
        a >>= 1;
    }

    return res;
}

// Returns (a**e) % m if a < m && e < m
static inline ulong modular_pow64(ulong a, ulong e, ulong m) {
    ulong sqr = a, acc = 1;

    while (e) {
        if (e & 1) acc = modular_add64(acc, sqr, m);
        sqr = modular_add64(sqr, sqr, m);
        e >>= 1;
    }

    return acc;
}

// Returns a random float from [0,1] based off the state and the global_id (distance)
static inline float random(
    const uint2 state,
    const uint distance
) {
    const ulong A = 4294883355;
    const ulong M = 18446383549859758079;

    const ulong m = modular_pow64(A, distance, M);
    ulong x = (ulong) state.x * A + (ulong) state.y;
    x = modular_mul64(x, m, M);
    x = (x / A) ^ (x % A);

    return (float) ((double) x * 2.3283064370807974e-10);
}

//#################################################################################################
//
//                                             Kernels
// 
//#################################################################################################

// Apply the gate [[u00, u01], [u10, u11]] to the #target qbit of the buffer
kernel void apply_gate(
    global float2 *buffer,
    const uchar size,
    const uchar target,
    const float2 u00,
    const float2 u01,
    const float2 u10,
    const float2 u11
) {
    const size_t i = get_global_id(0);

    const uchar r = size - target - 1;
    
    const size_t pow = (size_t) 1 << r;
    const size_t div = i >> (r + 1);

    const size_t j = (div * (2 << r)) + (i & (pow-1));
    const size_t k = j + pow;

    if ((i & ((pow << 1)-1)) < pow) {
        buffer[i] = complex_mul(u00, buffer[j]) + complex_mul(u01, buffer[k]);
    } else {
        buffer[i] = complex_mul(u10, buffer[j]) + complex_mul(u11, buffer[k]);
    }
}

// Apply the gate [[u00, u01], [u10, u11]] to the #target qbit of the buffer
// with qbit #control as control 
kernel void apply_controlled_gate(
    global float2 *buffer,
    const uchar size,
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
    const uint2 state
) {
    const size_t global_id = get_global_id(0);
    const float rand = random(state, global_id);

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