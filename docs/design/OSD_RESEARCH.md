# Liquid Glass OSD: research note

Written 2026-11-27 for the rebuild of the OSD on the optical glass (`fh6_kit::OpticalGlass`, [docs/specs/optics.md](../specs/optics.md)). Every row marked **Cited** was read from the page named, on that date, by a fetch of the page itself (no search snippets). **Our reading** marks a conclusion drawn for this overlay; **derived** marks arithmetic on cited numbers. The material itself (lensing, materialising, highlights, merging) is already researched for the pilot in [the Liquid Glass research note](../../crates/fh6-clusters/src/clusters/liquid_glass/RESEARCH.md) §2 (A1 to A16); this note adds what a controller-driven settings screen needs: where glass belongs in a list of settings, how focus looks on a TV, how big things must be at a distance, and how CarPlay Ultra lets a driver personalise a cluster.

## 1. The bar

An Apple-grade settings overlay in the Liquid Glass design language, driven by a gamepad on a 55" 4K OLED, over a paused Forza frame. The closest Apple product is **tvOS 26** (Liquid Glass, focus-driven, remote or game controller, ten-foot viewing); the closest automotive product is **CarPlay Ultra**'s cluster personalisation. Neither is a racing game, so where they disagree with an overlay on an OLED, the OLED wins (§6).

## 2. Liquid Glass in a settings screen

| # | What the sources say | Source |
|---|---|---|
| L1 | **Glass is the navigation layer, not the content.** Liquid Glass "forms a distinct functional layer for controls and navigation elements" that "floats above the content layer". "Don't use Liquid Glass in the content layer." | Cited: HIG Materials |
| L2 | **Lists stay content.** Making a table view glass "would make it compete with other elements", so it stays in the content layer. Lists, tables and forms get "a larger row height and padding"; section headers "no longer render entirely in capital letters". | Cited: WWDC25 219 (Principles); Adopting Liquid Glass (Organization) |
| L3 | **A control becomes glass while it is used.** "The knob transforms into Liquid Glass during interaction." Sliders and toggles in the content layer take "a Liquid Glass appearance to emphasize its interactivity when a person activates it." | Cited: Adopting Liquid Glass (Controls); HIG Materials |
| L4 | **On tvOS, focus is the activation.** "Standard buttons and controls take on a Liquid Glass appearance when focus moves to them." | Cited: HIG Materials (tvOS) |
| L5 | **It materialises by lensing.** "Instead of fading, Liquid Glass objects materialize in and out by gradually modulating the light bending and lensing." | Cited: WWDC25 219 (Dynamics) |
| L6 | **It flexes and lights from within.** It "responds to interaction by instantly flexing and energizing with light", with "an inherent gel-like flexibility". "The material illuminates from within as a form of feedback", and the glow spreads to nearby glass. | Cited: WWDC25 219 (Dynamics, Principles) |
| L7 | **It morphs between controls.** "Liquid Glass dynamically morphs between" controls in each context; "buttons fluidly morph into menus and popovers"; a menu "simply pops open" as a bubble. | Cited: WWDC25 219; Adopting Liquid Glass |
| L8 | **It adapts to what is behind it.** "As text scrolls underneath, shadows become more prominent"; "the amount of tint and the dynamic range shift to always ensure buttons remain legible". Small glyphs flip light or dark; menus and sidebars do not. | Cited: WWDC25 219 (Adaptivity, Principles) |
| L9 | **Two variants, never mixed.** Regular "blurs and adjusts the luminosity of background content"; clear "is highly translucent" and needs a dimming layer: "if the underlying content is bright, consider adding a dark dimming layer of 35% opacity." | Cited: HIG Materials; WWDC25 219 |
| L10 | **No glass on glass, and sparingly.** "Always avoid glass on glass"; on glass use "fills, transparency, and vibrancy". "Avoid overcrowding or layering Liquid Glass elements on top of each other." "Use Liquid Glass effects sparingly." | Cited: WWDC25 219; Adopting Liquid Glass; HIG Materials |
| L11 | **Highlights follow geometry and movement.** "Light sources ... shine on the material producing highlights that respond to geometry"; "in some cases, the lighting responds to device motion". | Cited: WWDC25 219 (Principles); Apple Newsroom, June 2025 |
| L12 | **Concentric rounded shapes.** Controls use "rounded shapes that are concentric to their containers"; "sections have an increased corner radius". No radius values are published. | Cited: Adopting Liquid Glass |
| L13 | **Edges of a scroll dissolve.** The scroll edge effect "gently dissolves the content into the background"; over dark content it "switches to apply a subtle dimming instead". | Cited: WWDC25 219 (Adaptivity); HIG Materials |
| L14 | **Accessibility changes the material.** Reduced Transparency "makes Liquid Glass frostier"; Increased Contrast "makes elements predominantly black or white" with "a contrasting border"; Reduced Motion "disables any elastic properties". | Cited: WWDC25 219 (Principles) |

**Not stated anywhere read:** games, HUDs, gamepads or overlays; the exact behaviour of a segmented control's or slider's indicator; any radius, opacity or duration value.

## 3. Focus and the ten-foot screen

| # | What the sources say | Source |
|---|---|---|
| T1 | **Focus is depth, light and motion.** The focused item "stands out ... through elevation to the foreground, illumination, and animation"; "tvOS generally uses the parallax effect"; focusing "often increases its scale". | Cited: HIG Focus and selection |
| T2 | **No pointer; every element reachable; focus never jumps.** "Avoid displaying a pointer." People must be able "to bring focus to every element". "Avoid changing focus without people's interaction"; when a focused item disappears, focus moves only to a nearby item. Selecting is a separate gesture from focusing. | Cited: HIG Focus and selection |
| T3 | **tvOS 26 glass on controls.** tvOS 26 simulates "refractions and reflections to create the illusion of specular highlights", "adding a subtle three-dimensionality to every control, button, and app icon border". Written about the developer beta; the article warns the release may differ. | Cited: MacStories tvOS 26 overview |
| T4 | **Android TV focus numbers.** "Default scaling values are: 1.025, 1.05 and 1.1x"; a glow under the element from 2 to 32 dp; an outline "independent from the border"; the pressed state lasts "until the remote button is released"; "surface color can change when focused". | Cited: Android TV focus system |
| T5 | **Ten-foot sizes (Xbox, 1080p at 200%).** "Main text and reading content: 15 epx minimum"; "Non-critical text and supplemental content: 12 epx minimum"; "Interactive UI elements should be sized at a minimum height of 32 epx." | Cited: Designing for Xbox and TV (archived UWP) |
| T6 | **Reach.** From one edge of the screen to the other "should take no more than six clicks". Tooltips are avoided on TV. | Cited: Designing for Xbox and TV |
| T7 | **Title safe.** "For HDTV modes, the title-safe area is the inner 90 percent"; "draw only non-essential visuals within 5% of the screen edges." | Cited: Introduction to the 10-foot experience; Designing for Xbox and TV |
| T8 | **Colour on a TV.** RGB 16 to 235 "are generally safe to use for TV"; high-intensity colours "may cause blooming"; "Don't assume colors will look exactly as they do on your monitor"; Xbox apps default to the dark theme. | Cited: Designing for Xbox and TV |
| T9 | **D-pad grammar.** "Users can use arrow keys to move, center button to select, and back key to return." XY focus moves only up, down, left and right. | Cited: Android TV focus system; Designing for Xbox and TV |

**Derived for this panel** (3840 × 2160 on a 55" panel, 1 px = 0.317 mm, from [RESEARCH.md](RESEARCH.md)): one epx on Xbox is 2 px at 1080p, so 4 px at 4K.
- Main text: 15 epx = 60 px of font size at 4K, about **42 px of cap height** for a face whose capitals are 0.7 em. Our labels are 30 px cap, 71% of that, so they meet the Xbox minimum at **2.2 m or nearer** (3.05 m × 30 / 42). At the rig distance of 0.8 m they are 3.8 times it.
- Secondary text: 12 epx = 48 px, about **34 px cap**. Our help and hint lines (26 px cap) meet it at 2.3 m or nearer.
- Interactive height: 32 epx = **128 px**. Our rows are 116 px, the focused row 148 px.
- Title safe: 5% of the frame is **192 px at the sides and 108 px at the top and bottom**. Our values end at x 3552 and the panel at 3700, so the text is inside it and only the glass edge is not.

**Our reading:** the LG C1 in PC mode shows every pixel, so title safe is a courtesy, not a crop. The viewing distance is not recorded anywhere in the repo (the dash research assumes 0.8 m at the rig; the OSD prompt says "the couch"), so the OSD is sized to pass at 2.2 m and is checked at both.

## 4. CarPlay Ultra: how a driver personalises the cluster

| # | What the sources say | Source |
|---|---|---|
| C1 | **Curated themes.** "A handful of curated gauge and information cluster themes"; Aston Martin offers "classic twin dial or innovative 'Power Bar' themes". | Cited: InsideEVs; Aston Martin |
| C2 | **Colours and backgrounds on top.** Drivers "choose between colours, textures and patterns, including gradients, landscapes and topographical options"; Apple's themes "can be further embellished with different colors and wallpapers". | Cited: Aston Martin; 9to5Mac |
| C3 | **Layouts and widgets.** Colours and backgrounds apply "on a variety of layouts"; widgets show trip duration, fuel economy, calendar and weather, on either screen. | Cited: Motor1; Aston Martin; TechRadar |
| C4 | **Chosen from the wheel.** Theme options and the centre information "can be scrolled through using the small touchpad on the steering wheel" (DBX707 hands-on). | Cited: 9to5Mac |
| C5 | **Made per brand.** Aston Martin "developed its own bespoke CarPlay Ultra theme"; each car downloads "a folder full of graphical assets specific to each vehicle". | Cited: 9to5Mac; Motor1 |
| C6 | **It looks like Apple and it is quick.** "Apple's characteristic flat and simple graphic design choices"; "It's all undeniably Apple"; it "ran faster and looked slicker" than the car's own system; responses were "immediate and free of any unsightly hiccups or choppy transitions" (The Verge, quoted). | Cited: InsideEVs; Motor1; 9to5Mac |
| C7 | **Alerts in place.** "Alerts such as tyre pressure warnings appear as pop-ups in situ." | Cited: Aston Martin |

**Not stated anywhere read:** whether personalisation happens on the car's screen or the iPhone, whether the cluster previews a choice live, whether CarPlay Ultra uses Liquid Glass, its typography, and how focus or selection is drawn for the wheel touchpad.

**Our reading:** the flow to match is theme first, then colour, then fine-tuning, all from a control on the wheel, with no lag. Our OSD already has that order (Cluster carousel, Colour, Type & logo, Fit & camera) and goes further on one point the sources do not claim for CarPlay: the real cluster previews every change live beside the menu.

## 5. Why the current OSD reads as a slab

From `crates/fh6-osd/src/draw.rs` as of `2f85973`, and the pilot's own diagnosis (pilot research §4):

| Fault | Cause | Breaks |
|---|---|---|
| One big glass slab | The whole column (1320 × 1880 px, radius 56) is one legacy `Glass` plate over a 0.62 black backing fill. Glass used as a background panel. | L1, L2, rule 7 |
| Tinted, frosted, painted | Tint alpha 64, frost 0.9 with a σ 28 backdrop blur, a painted crown sheen and painted rims, the legacy material AGENTS.md 3A now forbids. | L9, 3A mandate |
| The rows live inside glass | Every row, its focus ring and its controls sit on the slab, so the focus lift is glass on glass in spirit. | L10 |
| Focus is a ring | A white 0.07 lift, a 3 px accent rim and a σ 14 glow. It reads, but it is a painted outline, not elevation and light. | T1, L4 |
| Nothing is bent | The slab sits over the dimmed game and its own black backing, so its lens has little to bend. | L5 (pilot P3) |
| Motion fades | Open fades and slides 64 px; the tab content crossfades. Nothing materialises. | L5 |

## 6. What the rebuild takes, and what differs here

| # | Principle | From |
|---|---|---|
| P1 | **No panel.** The rows are content: light on true black, with a feathered black pool behind the column instead of a slab. | L1, L2, rule 7 |
| P2 | **Glass marks only what you are on.** The focused row, the selected tab and the control being changed are the glass; everything else is light on black. | L3, L4, L10 |
| P3 | **Focus is a lens that lifts the row.** The focused row's content grows slightly (Android's middle step, 1.05) under a clear glass platter whose bevel bends the light under it, and the platter lights from within. No ring. | T1, T4, L4, L6 |
| P4 | **It moves like liquid.** Focus and tab lenses travel with a stretch (a leading and a trailing edge on separate springs) and settle; the platter morphs between row heights. | L6, L7 |
| P5 | **It materialises.** Opening ramps the glass thickness from zero, so lensing and highlights grow in; closing stays one frame. | L5, pilot P9 |
| P6 | **One material with the cluster.** The OSD's glass is the pilot's clear sapphire under the same studio, so the menu and the Liquid Glass cluster beside it look like one material. | L11, pilot P6 |
| P7 | **Clear variant over our own dimming.** The game is dimmed by the kit underlay; the glass adds no frost and no tint. | L9, pilot P10 |
| P8 | **Text on glass is light, not glass.** Labels under a platter sit under its flat face, where it does not bend them. | L10, pilot P8 |
| P9 | **Concentric shapes.** The platter is concentric with the column's corner radius; tabs and keys are capsules. | L12 |
| P10 | **Scroll edges dissolve into black.** Rows fade out at the viewport's edges rather than being clipped. | L13 |
| P11 | **Reduced flash calms the motion too.** There is no reduced-motion setting; as in the pilot, the elastic stretch and the inner light's pulse turn down with the existing `ReducedFlash` setting (the Limiter light row's "Gentle pulse"). | L14 |
| P12 | **Sized for a distance.** Labels stay at least 30 px cap (Xbox minimum at 2.2 m or nearer), help at least 26 px cap, rows at least 128 px tall. | T5, §3 derived |
| P13 | **Highlights small, fields dark.** No large bright field; the brightest things are thin rims and the focused text. | T8 (blooming) |

**Differences from Apple that we keep on purpose (our reading):**
- Apple's regular glass blurs what is behind it. The overlay cannot blur a game it does not capture, and on OLED a frosted fill reads as a grey slab, so the OSD uses the clear variant only.
- Apple adapts tint and switches light or dark by sampling content. The OSD sits over a dimmed game and true black, so it stays in its dark state.
- tvOS lifts focus with a drop shadow. On true black a shadow is invisible, so elevation comes from scale, the platter's rim and its inner light.

## 7. Sources

Apple:
- Human Interface Guidelines, Materials: https://developer.apple.com/design/human-interface-guidelines/materials
- Human Interface Guidelines, Focus and selection: https://developer.apple.com/design/human-interface-guidelines/focus-and-selection
- "Adopting Liquid Glass": https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass
- WWDC25 session 219, "Meet Liquid Glass": https://developer.apple.com/videos/play/wwdc2025/219/
- Apple Newsroom, "Apple introduces a delightful and elegant new software design" (June 2025): https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/

TV and controllers:
- MacStories, "tvOS 26: The MacStories Overview": https://www.macstories.net/news/tvos-26-the-macstories-overview/
- Microsoft, "Introduction to the 10-foot experience for Windows game developers": https://learn.microsoft.com/en-us/windows/win32/dxtecharts/introduction-to-the-10-foot-experience-for-windows-game-developers
- Microsoft, "Designing for Xbox and TV" (archived UWP page, English text): https://learn.microsoft.com/pl-pl/windows/apps/design/devices/designing-for-tv
- Android Developers, "Focus system" (TV): https://developer.android.com/design/ui/tv/guides/styles/focus-system

CarPlay Ultra:
- 9to5Mac, CarPlay Ultra hands-on (2025-06-30): https://9to5mac.com/2025/06/30/carplay-ultra-hands-on-reveals-its-standout-best-new-feature/
- Motor1, Apple CarPlay Ultra review (2025-06-30): https://www.motor1.com/news/764192/apple-carplay-ultra-review/
- Aston Martin, "Aston Martin and Apple CarPlay Ultra": https://www.astonmartin.com/en/our-world/brand-stories/aston-martin-and-apple-carplay-ultra
- TechRadar, CarPlay Ultra's Aston Martin debut (2025-05-15): https://www.techradar.com/vehicle-tech/hybrid-electric-vehicles/apple-carplay-ultra-finally-rolls-out-here-are-4-things-you-need-to-know-about-its-aston-martin-debut
- InsideEVs, CarPlay Ultra launch (2025-05-15): https://insideevs.com/news/759736/carplay-ultra-aston-martin-launch/

Limits of this pass: the Microsoft UWP page is archived and its last 8k characters came back summarised, so only quotes from its first 40k are used. The Verge hands-on that 9to5Mac quotes was not fetched. No frame grab was measured, so no pixel value in the design comes from Apple or CarPlay footage.
