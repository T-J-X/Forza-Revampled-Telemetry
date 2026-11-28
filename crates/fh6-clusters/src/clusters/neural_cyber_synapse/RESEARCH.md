# NeuralCyberSynapse: research note

Build #37, catalog row 23, category Sci-Fi, Target `concept`, family Sci-Fi (FUI / film dialect), view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used throughout. **Cited** means read on 2026-11-28 in the source linked (section 10 lists every URL). **Computed** means worked out here with `python3 -I` and numpy from cited equations; inputs and outputs are given so step 1 can re-run them. **Our reading** is a conclusion for this dash, open to the direction check. **Unconfirmed** means it was not read in a source today and step 1 must check it.

## 1. The benchmark

A `concept` dash names one real object for the visual review. **Ours is the all-optical voltage-imaging map of a spike back-propagating through the dendritic tree of a single CA1 pyramidal neuron**, from Park, Wong-Campos et al., "Dendritic excitations govern back-propagation via a spike-rate accelerometer" (Nature Communications 2025; open preprint on PMC), Fig. 1d to 1f and Fig. 2g. It is a real neural imaging display that shows exactly the brief's gesture: a spike leaves the soma and climbs a dendritic tree, frame by frame, on black. It replaces the brief's vaguer "confocal micrographs of Purkinje neurons" as the benchmark, for a physical reason given in B5.

What makes it recognisable (each **Cited**, PMC10312650, Fig. 1 and 2 captions and text):

| # | Feature | Note |
|---|---|---|
| B1 | A grey two-photon structural image of one neuron is the base layer; the voltage signal is overlaid on it as a colour amplitude map. | Fig. 1c, 1d. The structure is always there, dim; only the signal is bright. |
| B2 | Three views of the same spike: an **amplitude map** (peak ΔF per pixel), a **delay map** (when the spike arrived), and a sub-frame interpolation of the wavefront at 25 µs resolution. | Fig. 1d to 1f, SNAPT method. |
| B3 | A **kymograph**: voltage along the apical trunk as a heat map, soma at the top, distal dendrite at the bottom, time across; a spike is a slanted band whose slope is its finite back-propagation velocity. | Fig. 2g. |
| B4 | The back-propagating spike's amplitude **decays exponentially** along the trunk, length constant 255 ± 10 µm (n = 93 spikes). When a dendritic spike (dSpike) fires, a **second zone of amplification** appears distally: distal amplitude is bimodal. | Fig. 2e, 2f, 2h. |
| B5 | dSpikes arise "if and only if the distal dendrites had been depolarized for at least 15 ms prior to arrival of a bAP"; the network acts as a spike-rate accelerometer: a burst after silence penetrates, a steady train fades. | Text after Fig. 3; title. |

Supporting physics for the gesture (each **Cited**):
- Stuart and Sakmann (Nature 367, 1994): action potentials "are initiated first in the axon and then actively propagate back into the dendritic tree" of neocortical pyramidal cells.
- Stuart, Schiller and Sakmann (J Physiol 505, 1997): back-propagation is "decremented" and frequency dependent; half-width grows and rate of rise falls with distance; the peak travels at about 0.5 m/s.
- Larkum, Zhu and Sakmann (Nature 398, 1999): one back-propagating action potential that coincides with distal input "within a time window of several milliseconds" triggers a dendritic calcium spike and a burst (BAC firing); inhibitory distal input can block it.
- **Purkinje cells do not do this.** The Commons figure adapted from Llinás and Sugimori shows that "at increasing distances from the soma the fast spikes are reduced in amplitude and are barely noticeable in the more peripheral recordings". A soma-to-crown spike is therefore pyramidal physics, not Purkinje physics.

**Our reading.** Keep the brief's strictly symmetric planar fan (it reads faster at speed than a real apical tuft), describe it on the board as a stylised apical tree, and drive it with the pyramidal back-propagation behaviour above. The Purkinje confocal image and Cajal's 1899 drawing stay as look references only: glowing single-hue processes on black, the soma brightest, fibres tapering to hairline. The benchmark's heat-map colours are replaced by luminance of the palette primary (section 9, item 6).

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Benchmark: spike amplitude map, delay map and sub-frame wavefront over the 2P structure (Fig. 1d to 1f) | not known (PMC figure viewer) | https://pmc.ncbi.nlm.nih.gov/articles/PMC10312650/ |
| 2 | Benchmark: kymograph of bAPs with and without a dSpike, soma at top (Fig. 2g), and amplitude profiles (Fig. 2h) | not known | same page, Fig. 2 |
| 3 | Confocal tdTomato Purkinje cells, glowing processes on black (CC BY 4.0) | https://upload.wikimedia.org/wikipedia/commons/6/6e/All_that_glitters_in_the_brain.jpg | https://commons.wikimedia.org/wiki/File:All_that_glitters_in_the_brain.jpg |
| 4 | Cajal 1899, Purkinje (A) and granule (B) cells, pigeon cerebellum (public domain) | https://upload.wikimedia.org/wikipedia/commons/1/15/PurkinjeCell.jpg | https://commons.wikimedia.org/wiki/File:PurkinjeCell.jpg |
| 5 | Somatic vs dendritic spikes in Purkinje dendrites (after Llinás and Sugimori, CC BY-SA 2.5) | https://upload.wikimedia.org/wikipedia/commons/2/29/Purkinje_cell_dendritic_spike.png | https://commons.wikimedia.org/wiki/File:Purkinje_cell_dendritic_spike.png |
| 6 | L5 pyramidal neuron and BAC firing (after Giugliano et al., CC BY-SA 2.0) | https://upload.wikimedia.org/wikipedia/commons/7/7b/L5_pyramidal_neuron_and_BAC_firing.png | https://commons.wikimedia.org/wiki/File:L5_pyramidal_neuron_and_BAC_firing.png |
| 7 | Animated reconstructed Purkinje morphology, NeuroMorpho NMO_00610 (CC BY-SA 3.0) | https://upload.wikimedia.org/wikipedia/commons/e/ea/Purkinje_cell_model.gif | https://commons.wikimedia.org/wiki/File:Purkinje_cell_model.gif |
| 8 | FitzHugh–Nagumo phase plane, a = 0.7, b = 0.8, τ = 12.5 (animated) | https://upload.wikimedia.org/wikipedia/commons/4/45/Fitzhugh-nagumo_b_=_0.8.gif (computed from the file name; PC checks) | https://en.wikipedia.org/wiki/FitzHugh%E2%80%93Nagumo_model |
| 9 | Action potential graph with depolarisation, repolarisation, hyperpolarisation and labelled refractory periods (Fig. 1 and 2) | not known | https://books.lib.uoguelph.ca/human-physiology/chapter/action-potentials/ |
| 10 | Brainbow lead image (Lichtman and Sanes 2008): multicolour neurons, shown as the colour route we do not take | not known | https://en.wikipedia.org/wiki/Brainbow |
| 11 | Equivalent-cylinder dendritic tree diagrams (Fig. 4 and 5) | not known | https://sites.pitt.edu/~phase/bard/bardware/classes/passive2/node14.html |
| 12 | Izhikevich 2003 summary figure of neuron types and parameters | not known | https://www.izhikevich.org/publications/spikes.htm |

Drive `references` folder: https://drive.google.com/drive/folders/1ZP_zA8m5l6jDGq5_xlC_oy-wrhE9fjJq (holds `SOURCES.md`; no image bytes saved here, the PC fetches each URL listed).

## 3. How the real thing behaves, with motion notes

Real timings are milliseconds and micrometres; the dash maps them onto rpm and its own springs. Each state below gives the neuron fact and the dash behaviour.

| State | The neuron (source) | The dash (Our reading unless marked) |
|---|---|---|
| Start-up | The rig takes a structural 2P scan before any voltage map (Park, Fig. 1 methods, **Cited**). | Boot ≤ 1 s (TOKENS): the grey tree draws level by level from the soma (5 steps, 150 ms each); gear and speed live from frame 1. Ignition sweep only on the first `race_on` 0 to 1. |
| Idle | Resting potential about −70 mV (Guelph, **Cited**; StatPearls gives about −60 mV). Nothing propagates. | The front rests at the idle position near the trunk base; traces flat; nothing moves (idle is still). |
| Cruise | A steady spike train: repetitive bAPs slowly inactivate dendritic NaV and penetrate less (Park intro, **Cited**). | Front steady; the spike highlight at the front settles to its decayed amplitude e^(−x/λ). |
| Pull to the limiter | Rate acceleration after silence lets the bAP penetrate and trigger dSpikes (Park, **Cited**). The bAP peak broadens and slows with distance (Stuart 1997, **Cited**). | The front climbs root-outward with rpm; the 2-node spike at the front broadens and dims with level (table in section 4). Rising rpm (`rpm_rate` > 0) and throttle held ≥ 150 ms add the distal amplification (dSpike), section 7. |
| Limiter | BAC firing: coincidence of a bAP with distal input gives a long calcium plateau (Larkum 1999, **Cited**). | Boutons strobe 10 Hz at 50% duty; the crown holds a steady plateau; the soma membrane goes 3 px accent, steady (brief). |
| Gear change | After a spike: repolarisation, then a hyperpolarising undershoot below rest (Guelph, **Cited**); absolute then relative refractory period. | The front retreats to the new rpm with an after-hyperpolarisation trail (fibres briefly darker than rest). Vesicle burst 150 ms. A 250 ms refractory lockout stops a second burst on a gear bounce. |
| Braking | Inhibitory distal input "can selectively block" the calcium spike (Larkum 1999, **Cited**). | Brake above 5% removes the distal plateau and amplification at once; trace 2 (warning) rises. |
| Drift, lateral G | none | Light azimuth from felt lateral G (`light` spring), 2 px tree lean (brief), clamped. |
| Reverse, neutral | none | Glyph R or N in the soma; the front still follows rpm. |
| Shut-down, pause | none | The host hides the dash within a frame (AGENTS rule 9); no exit animation. |

Real scale (**Computed**): a bAP at 0.5 m/s crosses 400 µm of apical trunk in 0.8 ms; a squid-axon spike (section 4) lasts about 3 ms above threshold at 6.3 °C. The dash cannot show real time; its front uses the `ribbon` spring (0.070) so the tach never lags.

## 4. The maths and physics

**The catalog row** (Sci-Fi table): a travelling Nagumo front v(x, t) = 1 / (1 + exp((x − ct) / √(2D))), c = √(D/2)(1 − 2a), and Rall's 3/2 law d_parent^(3/2) = Σ d_daughter^(3/2). Its test values check:

| Value | Source | Result |
|---|---|---|
| Front speed, D = 1, a = 0.25 | **Cited** form: Coombes' lecture notes, §3 p. 9, v = 1/(1 + exp[−(x + ct)/√2]), c = ±(2α − 1)/√2 for D = 1 | **Computed** 0.35355. A finite-difference run of u_t = u_xx + u(1 − u)(u − a) (dx 0.1, 150 time units) gives 0.3532, within 0.1%. |
| Symmetric branch ratio | **Cited** 3/2 condition: Ermentrout's equivalent-cylinder notes (Rall), worked example d^(3/2) sums 1+1+1 = 3 | **Computed** 2^(−2/3) = 0.62996, so 0.6300 checks. |
| Front shape | derived from the row | **Computed** v(0) = 0.5; v(√2) = 0.26894; 10% to 90% width = 2 ln 9 √(2D) = 6.2147 for D = 1. |
| Front speed against a | row | **Computed** c = 0.70711, 0.56569, 0.35355, 0.14142, 0, −0.35355, −0.70711 for a = 0, 0.1, 0.25, 0.4, 0.5, 0.75, 1. The front stalls at a = 0.5 and **retreats** for a > 0.5. |

**A correction to the brief (not to the maths row).** The brief tapers the fibres "10 -> 2 px" over five levels (1, 2, 4, 8, 16). Rall's law with a 10 px trunk gives 10, 6.300, 3.969, 2.500, **1.575 px** (**Computed**). For 2 px terminals the trunk must be **12.70 px**; a plain geometric 10 to 2 taper needs a ratio of 0.6687 per level, which breaks the 3/2 law (Σd^(3/2) ÷ d_p^(3/2) = 1.094 instead of 1). The maths row stands; the brief's taper needs fixing (the lead reports it). Our reading: prefer 12.70 to 2.00 px, which keeps the brief's 2 px terminal hairline; a 12 px trunk ending at 1.89 px also obeys the law.

**Equal time per level (Computed, Our reading on the use).** If each level has the same electrotonic length (the equivalent-cylinder condition), physical length goes as λ ∝ √d, giving length ratios 1, 0.7937, 0.6300, 0.5000, 0.3969 (sum 3.3205). Over the brief's 1300 px path that is 391.5, 310.7, 246.6, 195.8 and 155.4 px (cumulative 391.5, 702.2, 948.8, 1144.6, 1300). Front speed also goes as √d, so the front spends the same time in every level: rpm fraction maps linearly to electrotonic distance X and to the level count, which is what a tach needs. Whether an active front passes a Rall-matched branch point without delay (geometric ratio 1) is **unconfirmed** here (Goldstein and Rall 1974 is the lead); step 1 checks.

**Spike highlight along the tree (Computed from B4).** Mapping the 1300 px path to 400 µm of trunk, the default (no dSpike) highlight amplitude e^(−x/255 µm) is 0.624, 0.429, 0.318, 0.251 and 0.208 at the ends of the five levels. A dSpike adds a distal second zone (Park Fig. 2h): MODEL.

**Why a front and not a pulse (Our reading).** The Nagumo bistable front has no recovery: once lit, a fibre stays lit, which is a bar-graph tach. A real action potential is a pulse with recovery and refractoriness, which is FitzHugh–Nagumo: v' = v − v³/3 − w + RI, τw' = v + a − bw (**Cited**, Wikipedia FHN, with a = 0.7, b = 0.8, τ = 12.5 in its figures). Use the front for the fill and an FHN-style recovery variable only for the 2-node spike and the after-hyperpolarisation trail.

**Reference neuron numbers** (for the board and labels, not for tests):
- Hodgkin–Huxley 1952 squid axon, space-clamped, 6.3 °C (**Computed**, standard HH rate equations from Wikipedia, gNa 120, gK 36, gL 0.3 mS/cm²): peak +41.0 mV, minimum −76.2 mV, half-width 1.47 ms. Paired-pulse test: no second spike at 3 to 5 ms even at 25 times threshold (absolute refractory between 2 and 3 ms), and a normal stimulus fires again only after 10 to 12 ms (relative refractory).
- HH cable, radius 238 µm, Ri 35.4 Ω cm, φ = 3^((T − 6.3)/10): **Computed** 12.32 m/s at 6.3 °C and **18.74 m/s at 18.5 °C**. Hodgkin and Huxley's own figures (18.8 m/s computed against 21.2 m/s measured) are **unconfirmed** here; step 1 checks PMC1392413, Part III.
- Mammalian textbook values (**Cited**, Guelph): rest −70 mV, threshold −55 mV, peak about +30 mV, the whole action potential within about 2 ms, all or nothing; StatPearls gives −60 mV to +40 mV.
- Izhikevich 2003 (**Cited**, Musacchio's summary of the paper): v' = 0.04v² + 5v + 140 − u + I, u' = a(bv − u), reset at 30 mV to c with u += d; regular spiking a, b, c, d = 0.02, 0.2, −65, 8. A cheap alternative for the soma's "firing" if the board wants one.
- Engine against neuron (**Computed**, Our reading): a four-stroke fires rpm ÷ 60 × cylinders ÷ 2 times a second: 26.7 Hz at 800 rpm on four cylinders, 400 Hz at 6,000 rpm on a V8, 1,500 Hz at 18,000 rpm on a V10. Real neurons top out near 1 ÷ refractory period, a few hundred Hz: an honest metaphor for the limiter.

**Unit tests to assert:** c(D = 1, a = 0.25) = 0.35355; c(a = 0.5) = 0; c(a = 0.75) = −0.35355; Rall ratio 0.62996; diameters from 12.70 px end at 2.000 px (or from 10 px at 1.575 px); length ratios √(d/d₀) with sum 3.3205; v(√2) = 0.26894; e^(−400/255) = 0.208.

## 5. Telemetry mapping

| Visual part | Input (`Signals` / packet) | Status | Transform |
|---|---|---|---|
| Front position in the tree | `rpm`, `idle_rpm`, `shift_rpm` | verified | X = X_tot × clamp((rpm − idle) ÷ (0.75 shift − idle)), `ribbon` spring; Nagumo sigmoid edge |
| Bouton shift strip (16, outside-in) | `rpm`, `shift_rpm`, per-gear thresholds | verified | 75% to 100% of shift rpm, 0 ms attack, ≤ 50 ms release, hysteresis 100 rpm |
| Limiter strobe | `rpm` ≥ limiter threshold | verified | 10 Hz, 50% duty, boutons only; reduced flash: 2 Hz breath |
| Soma glyph | `gear` | verified | R, N, 1 to 10+; TOKENS gear change |
| Speed | `speed`, `unit` | verified | Saira 200, cap 100, change only when |v − shown| ≥ 0.6 |
| Spike height at the front | `throttle` | verified | brief: amplitude × throttle; decays e^(−x/λ) by level (MODEL) |
| Distal plateau (dSpike) | `throttle` held ≥ 150 ms, `rpm_rate` > 0, `brake` < 5% | verified | a second amplification zone on levels 4 and 5 (MODEL) |
| Retreat speed after upshift | `gear` rising edge, `rpm` | verified | Nagumo c with a > 0.5, capped; AHP trail 150 ms |
| Input traces (rank 4) | `throttle`, `brake`, `steer` | verified | three 512-point polylines, scroll on `timestamp`, flat under constant input |
| Soma rim gap | `clutch` | decoded, offset confirmed at check C | gap angle = clutch × 90°; fallback: none (hide the gap) |
| Light azimuth, tree lean | `g_lat` (felt, −acceleration_x) | verified | `light` spring; lean ≤ 2 px |
| Membrane rim thickness at limiter | limiter state | verified | 1 to 3 px, accent, steady |
| Vesicle burst | `gear` rising edge | verified | 12 particles, 150 ms, 250 ms lockout |
| Optional micro labels | `power`, `torque` | power verified; torque decoded, offset confirmed at check C | torque fallback = power × 60 ÷ (2π rpm) |

### What the game cannot send

The benchmark is a lab rig, so the parts to replace are its signals, not car functions. Each gets a bold replacement driven by telemetry; one per part is **recommended**.

1. **Real membrane voltage.** **Recommended: the Nagumo front driven by rpm**, labelled MODEL on the dash. Second: an FHN pulse train at the soma whose rate is the engine firing rate rpm ÷ 60 × cylinders ÷ 2 (`cylinders` decoded, offset confirmed at check C; fallback 4), shown as a slow-down visual alias, never as a real frequency.
2. **Where synaptic input lands on the tree.** **Recommended: steer as lateralised distal input**: steering left raises the left half-crown's resting luminance by up to 6%, right the right (signed, sprung, idle still). Second: the four subtrees of level 2 take `tyre_combined_slip` FL, FR, RL, RR (decoded, offset confirmed at check C), so a sliding wheel "depolarises" its quarter of the tree.
3. **The optogenetic stimulus patch (blue light on a dendrite).** **Recommended: boost as the stimulus**: a soft patch at the trunk base whose radius grows with `boost` (verified). Second: `handbrake` (decoded, check C) as a brief stimulus mark at the crown.
4. **The grey 2P structural image.** **Recommended: the dark tree at alpha 22 (brief) over a ComfyUI neuropil defocus plate** (LIBRARY GAP below), parallaxed 0 / 2 px per g. Second: Rall-correct spines along the fibres from a computed strip.
5. **EEG brain waves** (the brief's word; an EEG records the scalp, not one neuron). **Recommended: keep the three input traces and call them patch traces**, like the soma recording under Park Fig. 1b. Second: **a kymograph of the front** (Park Fig. 2g): rpm history as a slanted band per pull, soma at the bottom, so each upshift is a visible diagonal.
6. **The microscope's scale bar and time stamp.** **Recommended: a scale bar of distance per second** (speed in m per 1 s, from `speed`), labelled in Chakra Petch micro type. Second: `current_lap` time as the time stamp (decoded, check C; fallback: a session timer from `timestamp`).
7. **Preparation temperature** (HH speeds scale with Q10 = 3). Second-tier idea: mean `tyre_temp` (decoded, check C) narrows the spike highlight as the tyres warm, φ = 3^((T − T_ref)/10), MODEL. No fallback; hide when unavailable.

## 6. Edge cases

- **Max rpm 0 or tiny, idle near max.** Span = max(0.75 shift − idle, 500 rpm); with max rpm under 1,000 the front and boutons stay dark and gear and speed still draw.
- **12,000+ rpm and 4,500 rpm diesels.** Fraction-based, so the tree is the same; only the per-gear thresholds change. The engine-rate idea (section 5, item 1) caps its visual alias at the frame rate.
- **EVs.** Idle 0, single gear: the front starts at the root; the gear glyph follows kit rules; no shift strobe if the car never reaches a limiter.
- **Reverse, neutral, gear above 10.** R and N in the soma; two digits fit the 260 px soma only if cap 210 Saira Condensed 800 is fitted to the circle (check on the board; scale to fit).
- **mph and km/h, hp and kW.** From `unit` and `power_unit`; the scale bar idea uses metres either way.
- **Limiter bounce (about 50 rpm a frame).** The front is sprung; boutons use 100 rpm hysteresis; the strobe latches for 150 ms; the plateau holds 300 ms after the last limiter frame.
- **Spikes and crashes (huge G), airborne.** Lean and light clamp; the vesicle burst is gear-only, never G; airborne free-revving simply moves the front.
- **Stale or paused stream.** The host hides the dash.
- **Palette extremes and random palettes.** The front is primary lifted to white; if primary is already near white, the front keeps width (2-node spike) and the plateau uses accent. A red primary hands structure over to accent so red stays reserved (as in `F35HelmetHmd`).
- **Font swap.** The soma glyph is fitted to the 260 px circle whatever the display font.
- **Cockpit only.** No hood variant is listed.
- **Reduced flash.** Strobe becomes a 2 Hz breath; the vesicle burst becomes a 150 ms rim brightening; the AHP trail stays (it is a dimming).
- **Colour-only cues and colour vision.** Shift stages are also a count (outside-in); brake is trace 2 by position; the dSpike plateau also widens the fibres by 1 px; the front is a position.

## 7. Signature moments

**Limiter / shift strobe** (only the bouton strip strobes):
- A. The brief: boutons strobe at 10 Hz, soma membrane thickens to 3 px in accent, steady.
- B. **Recommended: BAC plateau.** As the boutons fill, the crown (levels 4 and 5) settles into a steady plateau glow in accent at ≤ 70% luminance (Larkum's calcium plateau), held while throttle stays on, while only the bouton strip strobes. Braking erases it instantly (inhibition blocks the calcium spike). Keep A's steady 3 px membrane.
- C. Refractory hold: the front stops one node short of the crown and the last node shows a dim "refractory" band, a still sign that the cell is saturated.

**Gear change:**
- A. The brief: 12 vesicles burst from the soma rim for 150 ms.
- B. **Recommended: front retreat with after-hyperpolarisation.** On a rising gear edge the front runs back to the new rpm at the Nagumo retreat speed (a > 0.5, capped at the `snap` spring), and the fibres it leaves dip below rest (alpha 22 to 12) for 150 ms before recovering, like the undershoot after a real spike. Keep A's burst as the event, with a 250 ms refractory lockout. The soma glyph change follows TOKENS.
- C. The kymograph (section 5, item 5) gains a diagonal band per shift, a record of each pull.

## 8. What it needs from the 4K library

From the pre-build kit (PR #18 branch, `kits.rs` and `assets/clusters/neural_cyber_synapse/prebuild/manifest.json`):
- `soma_membrane_grain` (computed, seamless 4096² master, seed 370001): faint fibres and 34 px granules, grey 0.38 to 0.6, with height and normal: optional modulation inside the 260 px soma; a gradient is the fallback. No sprites, swatches or ComfyUI textures were pre-built.

Shared parts it can reuse: `shift_led_32_off` / `_on` (SimInputTrace, SimPitWall) as a stand-in for the bouton caps if the board accepts 32 px; existing family plates in `assets/scifi/` (dot grids at 2k only, `plate_energy_core.png`) as references, not as runtime 4K assets; Saira, Saira Condensed and Chakra Petch fonts in `assets/fonts/`.

The asset rules ask for at least one bespoke ComfyUI and one bespoke Blender asset, both visibly used. Still needed:

LIBRARY GAP: `synaptic_bouton_30` off and on, Blender sprite, 30 px at 1:1 (2x master), rest plus the two G-swing keys: a glassy translucent bulb on a short neck, tinted by the kit; the 16 boutons are the shift strip and must read as rendered objects. Blender hero library.

LIBRARY GAP: `soma_body_260`, Blender sprite, 260 px (the brief's size): a thin translucent membrane shell with subsurface scatter and a lit 1 px rim, dark interior so the gear keeps 7:1; rest plus two G-swing keys. Blender hero library.

LIBRARY GAP: `neuropil_defocus_4096`, ComfyUI (SDXL base and refiner), seamless 4096² greyscale: out-of-focus fluorescent neuropil (blurred processes and puncta, like a confocal stack's haze), luminance ≤ 6% after tint, tinted by the palette: the depth behind the tree. Dressing only, never the figure.

LIBRARY GAP: `dendrite_spine_strip`, computed (CPU, seeded), 4096 × 128 seamless alpha strip of stubby and mushroom spines 1 to 3 px long, to edge the fibres at 1:1. asset-forge `prebuild` computed map.

LIBRARY GAP: `vesicle_16`, Blender sprite, 16 px: a clear membrane sphere with a specular dot, for the 12-particle upshift burst.

## 9. Departures and open questions for the board

1. **Neuron type.** The planar symmetric fan looks Purkinje, but Purkinje dendrites barely carry fast somatic spikes (B5 note); the physics is pyramidal. Recommended: stylised apical tree, pyramidal behaviour, Purkinje image for look only.
2. **Fibre taper.** 10 to 2 px breaks Rall's law; use 12.70 to 2.00 px or 10 to 1.575 px (section 4).
3. **Level lengths.** Recommend the √d lengths (391.5 to 155.4 px) so the front spends equal time per level; the board may scale the total.
4. **"EEG" renamed** to patch traces, or replaced by the kymograph (section 5, item 5).
5. **Saltatory conduction** (brief's references) belongs to myelinated axons, not dendrites: drop node beads from the tree; an axon below the soma is possible but would crowd ranks 1 and 2.
6. **Benchmark colours.** Park's maps use a heat-map LUT and Brainbow uses many hues; this Sci-Fi dash keeps palette roles: amplitude is luminance of primary, the plateau is accent, red reserved.
7. **Spike height.** The brief scales it with throttle; research adds decrement with distance and the dSpike second zone. Recommended: both, labelled MODEL.
8. **Customisable without losing the look** (TJS, 10:55, "you can change for example needle color etc"):

| Part | Maps to | Default |
|---|---|---|
| Fibre and front colour | palette `primary` (front lifted to white) | factory primary |
| Plateau and limiter membrane | palette `accent` | factory accent |
| Bouton stages | `go` / `warning` / `shift` roles | factory |
| Membrane rim and traces | `secondary`, `warning` for brake | factory |
| Gear and speed font | display and text font roles | Saira Condensed 800, Saira 200 |
| Dark-fibre alpha | `config.extra` | 22 |
| Neuropil plate brightness and on/off | `config.extra` | 6%, on |
| Spine detail on/off | `config.extra` | on |
| Rank-4 panel: patch traces or kymograph | `config.extra` | patch traces |

## 10. Sources

Benchmark and dendritic physiology
- Park, Wong-Campos et al., Dendritic excitations govern back-propagation via a spike-rate accelerometer (preprint v3, PMC): https://pmc.ncbi.nlm.nih.gov/articles/PMC10312650/
- Authors' summary: https://brain.harvard.edu/hbi_news/excitement-over-dendritic-excitations/
- Stuart and Sakmann 1994, Nature 367: https://www.nature.com/articles/367069a0
- Stuart, Schiller and Sakmann 1997, J Physiol 505: https://pmc.ncbi.nlm.nih.gov/articles/PMC1160040/
- Larkum, Zhu and Sakmann 1999, Nature 398: https://www.nature.com/articles/18686

Neuron models and the action potential
- Hodgkin–Huxley model (equations and rate functions): https://en.wikipedia.org/wiki/Hodgkin%E2%80%93Huxley_model
- Hodgkin and Huxley 1952 (to check at step 1): https://pmc.ncbi.nlm.nih.gov/articles/PMC1392413
- Squid giant axon: https://en.wikipedia.org/wiki/Squid_giant_axon
- FitzHugh–Nagumo model: https://en.wikipedia.org/wiki/FitzHugh%E2%80%93Nagumo_model
- Coombes, lecture notes, bistable front §3: https://www.maths.nottingham.ac.uk/plp/pmzsc/pdfs/coombes-1.pdf
- Izhikevich 2003: https://www.izhikevich.org/publications/spikes.htm and parameter summary https://www.fabriziomusacchio.com/blog/2024-04-29-izhikevich_model/
- Action potentials, Human Physiology (Guelph): https://books.lib.uoguelph.ca/human-physiology/chapter/action-potentials/
- StatPearls, Neuron Action Potential: https://www.ncbi.nlm.nih.gov/books/NBK546639/

Dendritic geometry
- Ermentrout, Equivalent cylinders (Rall's 3/2 rule): https://sites.pitt.edu/~phase/bard/bardware/classes/passive2/node14.html

Images
- https://commons.wikimedia.org/wiki/File:All_that_glitters_in_the_brain.jpg
- https://commons.wikimedia.org/wiki/File:PurkinjeCell.jpg
- https://commons.wikimedia.org/wiki/File:Purkinje_cell_dendritic_spike.png
- https://commons.wikimedia.org/wiki/File:L5_pyramidal_neuron_and_BAC_firing.png
- https://commons.wikimedia.org/wiki/File:Purkinje_cell_model.gif
- https://commons.wikimedia.org/wiki/Category:Purkinje_cells
- https://en.wikipedia.org/wiki/Brainbow

Repo
- `program/phases/07-cluster-factory/CATALOG.md` (brief, Sci-Fi maths row), `CLUSTER_PROMPT.md`, `docs/design/families/sci-fi.md`, `docs/design/TOKENS.md`, `docs/TELEMETRY_USAGE_MATRIX.md`, PR #18 branch `PREBUILD.md`, `kits.rs` and `assets/clusters/neural_cyber_synapse/prebuild/manifest.json`.
