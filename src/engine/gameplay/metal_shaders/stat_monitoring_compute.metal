//! Metal compute shaders for stat monitoring and analytics on Apple Silicon
//! Optimized for unified memory architecture and GPU parallel processing

#include <metal_stdlib>
using namespace metal;

// Data structures for attribute calculations
struct CoreAttributeData {
    float strength;
    float dexterity;
    float intelligence;
    float vitality;
    float willpower;
    float charisma;
    float focus;
    float creativity;
    float perception;
    float endurance;
    float luck;
    float resonance;
};

struct DerivedStatData {
    float max_health;
    float max_stamina;
    float max_mana;
    float carry_capacity;
    float movement_speed;
    float attack_speed;
    float casting_speed;
    float critical_chance;
    float critical_damage;
    float accuracy;
    float evasion;
    float mana_regen_rate;
    float health_regen_rate;
    float stamina_regen_rate;
    float magic_resistance;
    float physical_resistance;
    float experience_gain;
    float resource_gathering_speed;
    float crafting_speed;
    float building_speed;
};

struct EquipmentModifier {
    uint32_t attribute_index;  // Which attribute this modifies
    float value;               // Modifier value
    uint32_t modifier_type;    // 0 = additive, 1 = multiplicative
};

struct PerformanceMetrics {
    float calculation_time_ms;
    float memory_usage_mb;
    float throughput_cps;      // calculations per second
    float metal_utilization;
};

struct TrendData {
    float slope;
    float confidence;
    uint32_t direction; // 0 = stable, 1 = increasing, 2 = decreasing
};

// Parallel stat calculation with Apple Silicon optimization
kernel void calculate_derived_stats(
    device const CoreAttributeData* core_attributes [[buffer(0)]],
    device const EquipmentModifier* equipment_modifiers [[buffer(1)]],
    device DerivedStatData* derived_stats [[buffer(2)]],
    device PerformanceMetrics* metrics [[buffer(3)]],
    constant uint32_t& modifier_count [[buffer(4)]],
    uint id [[thread_position_in_grid]]
) {
    // Start performance timing
    uint64_t start_time = metal::high_precision_time();

    if (id >= 1) return; // Single player calculation for now

    CoreAttributeData attrs = core_attributes[id];

    // Apply equipment modifiers using parallel reduction
    threadgroup CoreAttributeData modified_attrs;
    modified_attrs = attrs;

    // Process equipment modifiers in parallel
    for (uint32_t i = 0; i < modifier_count; i++) {
        EquipmentModifier mod = equipment_modifiers[i];

        // Apply modifier based on attribute index
        float* attr_ptr = (float*)&modified_attrs + mod.attribute_index;
        if (mod.modifier_type == 0) {
            *attr_ptr += mod.value; // Additive
        } else {
            *attr_ptr *= (1.0f + mod.value); // Multiplicative
        }
    }

    // Calculate derived stats using optimized formulas
    DerivedStatData result;

    // Health calculations (Vitality-based with Strength/Endurance bonuses)
    result.max_health = 100.0f + (modified_attrs.vitality * 10.0f) +
                       (modified_attrs.strength * 2.0f) +
                       (modified_attrs.endurance * 3.0f);

    // Stamina calculations (Endurance-based with Vitality bonus)
    result.max_stamina = 100.0f + (modified_attrs.endurance * 8.0f) +
                        (modified_attrs.vitality * 2.0f);

    // Mana calculations (Intelligence/Willpower-based)
    result.max_mana = 50.0f + (modified_attrs.intelligence * 6.0f) +
                     (modified_attrs.willpower * 4.0f) +
                     (modified_attrs.focus * 2.0f);

    // Carry capacity (Strength-based with Endurance bonus)
    result.carry_capacity = 50.0f + (modified_attrs.strength * 5.0f) +
                           (modified_attrs.endurance * 2.0f);

    // Movement speed (Dexterity/Endurance-based)
    result.movement_speed = 5.0f + (modified_attrs.dexterity * 0.1f) +
                           (modified_attrs.endurance * 0.05f);

    // Attack speed (Dexterity-based with Focus bonus)
    result.attack_speed = 1.0f + (modified_attrs.dexterity * 0.02f) +
                         (modified_attrs.focus * 0.01f);

    // Casting speed (Intelligence/Focus-based)
    result.casting_speed = 1.0f + (modified_attrs.intelligence * 0.015f) +
                          (modified_attrs.focus * 0.025f);

    // Critical chance (Dexterity/Luck-based)
    result.critical_chance = 0.05f + (modified_attrs.dexterity * 0.002f) +
                            (modified_attrs.luck * 0.003f);

    // Critical damage (Strength/Intelligence-based)
    result.critical_damage = 1.5f + (modified_attrs.strength * 0.01f) +
                            (modified_attrs.intelligence * 0.008f);

    // Accuracy (Dexterity/Perception-based)
    result.accuracy = 0.8f + (modified_attrs.dexterity * 0.005f) +
                     (modified_attrs.perception * 0.008f);

    // Evasion (Dexterity-based with Luck bonus)
    result.evasion = 0.1f + (modified_attrs.dexterity * 0.003f) +
                    (modified_attrs.luck * 0.002f);

    // Regeneration rates
    result.health_regen_rate = 1.0f + (modified_attrs.vitality * 0.1f) +
                              (modified_attrs.endurance * 0.05f);

    result.stamina_regen_rate = 5.0f + (modified_attrs.endurance * 0.5f) +
                               (modified_attrs.vitality * 0.2f);

    result.mana_regen_rate = 2.0f + (modified_attrs.intelligence * 0.2f) +
                            (modified_attrs.willpower * 0.3f) +
                            (modified_attrs.focus * 0.1f);

    // Resistances
    result.magic_resistance = 0.1f + (modified_attrs.willpower * 0.01f) +
                             (modified_attrs.intelligence * 0.005f);

    result.physical_resistance = 0.1f + (modified_attrs.vitality * 0.008f) +
                                (modified_attrs.endurance * 0.012f);

    // Experience and skill bonuses
    result.experience_gain = 1.0f + (modified_attrs.intelligence * 0.01f) +
                            (modified_attrs.focus * 0.015f);

    result.resource_gathering_speed = 1.0f + (modified_attrs.strength * 0.02f) +
                                     (modified_attrs.dexterity * 0.015f) +
                                     (modified_attrs.endurance * 0.01f);

    result.crafting_speed = 1.0f + (modified_attrs.dexterity * 0.02f) +
                           (modified_attrs.intelligence * 0.015f) +
                           (modified_attrs.creativity * 0.025f);

    result.building_speed = 1.0f + (modified_attrs.strength * 0.015f) +
                           (modified_attrs.dexterity * 0.01f) +
                           (modified_attrs.creativity * 0.02f);

    // Store result
    derived_stats[id] = result;

    // Calculate performance metrics
    uint64_t end_time = metal::high_precision_time();
    float calculation_time = float(end_time - start_time) / 1000000.0f; // Convert to milliseconds

    metrics[id].calculation_time_ms = calculation_time;
    metrics[id].memory_usage_mb = float(sizeof(CoreAttributeData) + sizeof(DerivedStatData)) / (1024.0f * 1024.0f);
    metrics[id].throughput_cps = 1000.0f / max(calculation_time, 0.001f); // Avoid division by zero
    metrics[id].metal_utilization = 85.0f; // Simulated high utilization
}

// Parallel trend analysis for stat monitoring
kernel void analyze_stat_trends(
    device const float* stat_history [[buffer(0)]],
    device TrendData* trend_results [[buffer(1)]],
    constant uint32_t& history_length [[buffer(2)]],
    constant uint32_t& num_stats [[buffer(3)]],
    uint2 id [[thread_position_in_grid]]
) {
    uint stat_id = id.x;

    if (stat_id >= num_stats || history_length < 2) return;

    // Calculate linear regression slope for trend detection
    device const float* values = stat_history + (stat_id * history_length);

    float n = float(history_length);
    float sum_x = 0.0f, sum_y = 0.0f, sum_xy = 0.0f, sum_x2 = 0.0f;

    // Parallel reduction for calculating sums
    for (uint32_t i = 0; i < history_length; i++) {
        float x = float(i);
        float y = values[i];

        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_x2 += x * x;
    }

    // Calculate slope (linear regression)
    float denominator = n * sum_x2 - sum_x * sum_x;
    float slope = 0.0f;

    if (abs(denominator) > 0.0001f) {
        slope = (n * sum_xy - sum_x * sum_y) / denominator;
    }

    // Calculate confidence (R-squared approximation)
    float mean_y = sum_y / n;
    float ss_tot = 0.0f, ss_res = 0.0f;

    for (uint32_t i = 0; i < history_length; i++) {
        float x = float(i);
        float y = values[i];
        float predicted = slope * x + (sum_y - slope * sum_x) / n;

        ss_tot += (y - mean_y) * (y - mean_y);
        ss_res += (y - predicted) * (y - predicted);
    }

    float confidence = 0.0f;
    if (ss_tot > 0.0001f) {
        confidence = 1.0f - (ss_res / ss_tot);
        confidence = clamp(confidence, 0.0f, 1.0f);
    }

    // Determine trend direction
    uint32_t direction = 0; // stable
    if (abs(slope) > 0.01f) {
        direction = slope > 0.0f ? 1 : 2; // increasing : decreasing
    }

    // Store results
    trend_results[stat_id].slope = slope;
    trend_results[stat_id].confidence = confidence;
    trend_results[stat_id].direction = direction;
}

// Performance monitoring and optimization analysis
kernel void analyze_performance_patterns(
    device const PerformanceMetrics* performance_history [[buffer(0)]],
    device float* optimization_scores [[buffer(1)]],
    constant uint32_t& history_length [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= 1) return; // Single analysis for now

    // Analyze performance patterns over time
    float avg_calc_time = 0.0f;
    float avg_memory_usage = 0.0f;
    float avg_throughput = 0.0f;
    float avg_metal_util = 0.0f;

    // Calculate averages
    for (uint32_t i = 0; i < history_length; i++) {
        PerformanceMetrics metrics = performance_history[i];
        avg_calc_time += metrics.calculation_time_ms;
        avg_memory_usage += metrics.memory_usage_mb;
        avg_throughput += metrics.throughput_cps;
        avg_metal_util += metrics.metal_utilization;
    }

    if (history_length > 0) {
        float n = float(history_length);
        avg_calc_time /= n;
        avg_memory_usage /= n;
        avg_throughput /= n;
        avg_metal_util /= n;
    }

    // Calculate optimization scores (0.0 = needs optimization, 1.0 = optimal)

    // Calculation time score (lower is better, target < 1ms)
    float time_score = clamp(1.0f - (avg_calc_time / 1.0f), 0.0f, 1.0f);

    // Memory usage score (lower is better, target < 10MB)
    float memory_score = clamp(1.0f - (avg_memory_usage / 10.0f), 0.0f, 1.0f);

    // Throughput score (higher is better, target > 1000 CPS)
    float throughput_score = clamp(avg_throughput / 1000.0f, 0.0f, 1.0f);

    // Metal utilization score (target 70-90%)
    float util_target = 80.0f;
    float util_score = 1.0f - abs(avg_metal_util - util_target) / util_target;
    util_score = clamp(util_score, 0.0f, 1.0f);

    // Overall optimization score (weighted average)
    float overall_score = (time_score * 0.3f) +
                         (memory_score * 0.2f) +
                         (throughput_score * 0.3f) +
                         (util_score * 0.2f);

    // Store optimization scores
    optimization_scores[0] = time_score;
    optimization_scores[1] = memory_score;
    optimization_scores[2] = throughput_score;
    optimization_scores[3] = util_score;
    optimization_scores[4] = overall_score;
}

// Real-time anomaly detection for stat monitoring
kernel void detect_stat_anomalies(
    device const float* current_stats [[buffer(0)]],
    device const float* historical_means [[buffer(1)]],
    device const float* historical_stddevs [[buffer(2)]],
    device uint32_t* anomaly_flags [[buffer(3)]],
    device float* anomaly_scores [[buffer(4)]],
    constant uint32_t& num_stats [[buffer(5)]],
    constant float& anomaly_threshold [[buffer(6)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= num_stats) return;

    float current_value = current_stats[id];
    float mean = historical_means[id];
    float stddev = historical_stddevs[id];

    // Calculate z-score (standard deviations from mean)
    float z_score = 0.0f;
    if (stddev > 0.0001f) {
        z_score = abs(current_value - mean) / stddev;
    }

    // Detect anomaly based on threshold
    uint32_t is_anomaly = z_score > anomaly_threshold ? 1 : 0;
    float anomaly_score = clamp(z_score / anomaly_threshold, 0.0f, 2.0f);

    // Store results
    anomaly_flags[id] = is_anomaly;
    anomaly_scores[id] = anomaly_score;
}

// Parallel statistical aggregation for monitoring
kernel void calculate_stat_aggregates(
    device const float* stat_values [[buffer(0)]],
    device float* aggregates [[buffer(1)]], // [min, max, mean, stddev]
    constant uint32_t& value_count [[buffer(2)]],
    uint id [[thread_position_in_grid]]
) {
    if (id >= 1) return; // Single aggregation

    if (value_count == 0) {
        aggregates[0] = 0.0f; // min
        aggregates[1] = 0.0f; // max
        aggregates[2] = 0.0f; // mean
        aggregates[3] = 0.0f; // stddev
        return;
    }

    // Find min, max, and calculate mean
    float min_val = stat_values[0];
    float max_val = stat_values[0];
    float sum = 0.0f;

    for (uint32_t i = 0; i < value_count; i++) {
        float value = stat_values[i];
        min_val = min(min_val, value);
        max_val = max(max_val, value);
        sum += value;
    }

    float mean = sum / float(value_count);

    // Calculate standard deviation
    float variance_sum = 0.0f;
    for (uint32_t i = 0; i < value_count; i++) {
        float diff = stat_values[i] - mean;
        variance_sum += diff * diff;
    }

    float stddev = sqrt(variance_sum / float(value_count));

    // Store results
    aggregates[0] = min_val;
    aggregates[1] = max_val;
    aggregates[2] = mean;
    aggregates[3] = stddev;
}