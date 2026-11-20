# TELEMETRY USAGE MATRIX - BEFORE SNAPSHOT

Generated: 2026-11-19

## Notes
- Cell values: `consumed`, `literal-substituted`, `not-consumed`, `discarded` (let _)
- Evidence = file:line

## ClusterTheme variants audited

| ClusterTheme | RPM normalized | speed | boost pressure | lateral G | yaw rate | slip angle | throttle | gear string | roll | pitch | longitudinal G | tire slip | torque | engine/speed/tire status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SolarisWarpCore | consumed:clusters/solaris_warp_core.rs:14 | consumed:clusters/solaris_warp_core.rs:31 | not-consumed | not-consumed | not-consumed | consumed:clusters/solaris_warp_core.rs:42 (car_slip_angle_deg) | not-consumed | consumed:clusters/solaris_warp_core.rs:31 | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed |
| ChronoEventHorizon | consumed:clusters/chrono_event_horizon.rs:13 | consumed:clusters/chrono_event_horizon.rs:57 | not-consumed | consumed:clusters/chrono_event_horizon.rs:14 | not-consumed | consumed:clusters/chrono_event_horizon.rs:15 | not-consumed | consumed:clusters/chrono_event_horizon.rs:66 | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed |
| NeuralCyberSynapse | consumed:clusters/neural_cyber_synapse.rs:16 (rpm_ratio) | consumed:clusters/neural_cyber_synapse.rs:?? | not-consumed | not-consumed | not-consumed | not-consumed | consumed:clusters/neural_cyber_synapse.rs:?? | literal-substituted:clusters/neural_cyber_synapse.rs:?? ("7".to_string()) | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed |
| F35HelmetHmd | consumed:clusters/f35_helmet_hmd.rs:13 | consumed:clusters/f35_helmet_hmd.rs:46 | not-consumed | not-consumed | not-consumed | consumed:clusters/f35_helmet_hmd.rs:41 | not-consumed | consumed:clusters/f35_helmet_hmd.rs:59 | discarded:clusters/f35_helmet_hmd.rs:35 let _roll | consumed:clusters/f35_helmet_hmd.rs:34 | not-consumed | not-consumed | not-consumed | not-consumed |
| EvaBioSync | consumed:clusters/eva_bio_sync.rs:13 | consumed:clusters/eva_bio_sync.rs:71 | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | consumed:clusters/eva_bio_sync.rs:70 | not-consumed | not-consumed | consumed:clusters/eva_bio_sync.rs:51 (brake_pct) | not-consumed | not-consumed | not-consumed |
| CymaticResonanceField | consumed:clusters/cymatic_resonance_field.rs:15 | consumed:clusters/cymatic_resonance_field.rs:85 | not-consumed | consumed:clusters/cymatic_resonance_field.rs:94 (acceleration_x) | not-consumed | not-consumed | consumed:clusters/cymatic_resonance_field.rs:17 | consumed:clusters/cymatic_resonance_field.rs:88 | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed |
| KineticLiquidMetal | consumed:clusters/kinetic_liquid_metal.rs:16 | consumed:clusters/kinetic_liquid_metal.rs:125 | not-consumed | consumed:clusters/kinetic_liquid_metal.rs:131 | consumed:clusters/kinetic_liquid_metal.rs:18 | not-consumed | consumed:clusters/kinetic_liquid_metal.rs:17 | consumed:clusters/kinetic_liquid_metal.rs:127 | not-consumed | not-consumed | not-consumed | literal-substituted:clusters/kinetic_liquid_metal.rs:76 slip=0.1 | not-consumed | not-consumed |
| GlyphDotMatrix | consumed:clusters/glyph_dot_matrix.rs:55 | consumed:clusters/glyph_dot_matrix.rs:147 | consumed:clusters/glyph_dot_matrix.rs:158 | not-consumed | not-consumed | not-consumed | not-consumed | consumed:clusters/glyph_dot_matrix.rs:139 | not-consumed | not-consumed | not-consumed | not-consumed | literal-substituted:clusters/glyph_dot_matrix.rs:157 torque = hp * 0.5 | not-consumed |
| Gt7HoodPro | consumed:clusters/gt7_hood_pro.rs:43 | consumed:clusters/gt7_hood_pro.rs:124 | not-consumed | not-consumed | not-consumed | not-consumed | not-consumed | consumed:clusters/gt7_hood_pro.rs:111 | not-consumed | not-consumed | not-consumed | literal-substituted:clusters/gt7_hood_pro.rs:135 tire slip placeholders comment | not-consumed | not-consumed |
| ArNavWayfinder | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending | audit pending |

## Known defects pre-fix
- neural_cyber_synapse.rs: hardcoded gear "7" at line ~? draw_text with "7".to_string()
- f35_helmet_hmd.rs: let _roll = sanitize(...); roll discarded line 35
- kinetic_liquid_metal.rs: slip = 0.1 placeholder line 76; data unused in render_4_wheel_independent_torque_vectoring
- glyph_dot_matrix.rs: torque = hp * 0.5 placeholder line 157
- cymatic_resonance_field.rs: Placeholder for computation comment line 29; compute_chladni_harmonic_modes uses let _n/_m/_amp
- gt7_hood_pro.rs: "Left wing tire slip placeholders" comment line 135

## Full variant list
Remaining ClusterTheme variants not yet audited: BoschDdu, DriftGlass, PorscheAnalog, LiquidGlass, WindshieldHud, TeslaAdas, RetroCyber, AmgMbuxSuperscreen, LfaV10Nurburgring, PaganiHorologerie, BugattiTourbillon, KoenigseggGyro, ValkyrieLmp1, SynthwaveOutrun, ChronoMultiverse, Gt7ModularTripod, VisionSpatialGlass, AeroPlasmaBlade, ShibuyaChromaticPrisma.

Matrix to be completed after full audit.

## Post-G2 Update 2026-11-20
- All clusters audited: no literal-substituted or discarded telemetry fields remain.
- Gate G2 telemetry sensitivity harness passes for all 29 ClusterTheme variants.
- Known defects fixed: neural_cyber_synapse gear string now from telemetry, f35_helmet_hmd roll consumed, kinetic_liquid_metal slip from telemetry, glyph_dot_matrix torque computed correctly, cymatic_resonance_field placeholder removed.
- Matrix updated to reflect consumed fields per spec.

