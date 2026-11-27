# Materials, light and glass checks

Checklist line 18 means: every check below that applies to the parts your dash has is yes. Drop the ones for parts it does not have before you start, and say which. Each check names where to look and the fake it catches. Crop files are `mockups/<Name>/<state>_crop.png` (cockpit, read 1:1) or `hood_<state>_crop.png` (tile it); "3x" means `visual-review tiles ... --zoom 3` or `crop --zoom 3`. A 3x zoom shows detail no driver can see: use it to find a cause. If the only evidence for "it reads" is at 3x, the answer is no.

The physics behind the glass checks is `docs/specs/optics.md` (section numbers in brackets). Apple's own description of Liquid Glass is the WWDC25 session [Meet Liquid Glass](https://developer.apple.com/videos/play/wwdc2025/219/) and the [HIG on materials](https://developer.apple.com/design/human-interface-guidelines/materials). The project follows Apple's clear variant with local dimming, so frost is a fault.

## Glass

| # | Check (yes/no) | Look at | The fake it catches |
|---|---|---|---|
| G1 | Over the black ground, where nothing is drawn under it, is each glass piece visible only as thin light lines along its outline, with a black body? | 3x of every piece in `01_idle_crop.png` where nothing lit is under the glass | flat alpha posing as glass: a grey, navy or milky body, a stroke drawn on the outline, a soft glow band (optics §7, §8) |
| G2 | Is each outline brightest on the key-light side (upper left at rest), with a cooler arc opposite, and a second, shorter line further in on the key side? | 3x of the largest piece and one small pill, `02_mid_cruise_crop.png` | a rim of equal brightness all round, or a white-to-cyan gradient and sheen fixed to the screen (the legacy slab glass) |
| G3 | Are the rim lines narrow and crisp, about a pixel on a small pill and wider only on a big shallow dome, so the width follows each piece's size and curvature? | 3x of the smallest and largest piece on one sheet | thick or blurry rims; one stroke width everywhere (a painted outline) |
| G4 | Is content seen through the glass as sharp as the same kind of content outside it, changed only by bending? | 3x where a tick, the ribbon or a line passes under a piece | frost: blurred, grainy or milky content |
| G5 | Is the inside of every piece free of a picture of its own (caustic image, noise, sparkle, gradient)? | 3x of each interior in `01_idle_crop.png` against `03_redline_strobe_crop.png` | a texture painted inside the glass |
| G6 | Where a straight feature passes under a piece's edge, does it kink at the outline and curve across the bevel, then run straight under the flat middle? | 3x of an edge over a tick, the ribbon or a crosshair | lines running straight through (a decal), or everything warped by one radial distortion (optics §4) |
| G7 | Under a domed piece, is what sits beneath its centre shown larger than drawn, stretching more towards the rim? | 1:1 of the dome: visible cap height against the `Geom` value; 3x near the rim | a plain-size glyph with a ring round it, or an even resize |
| G8 | Does a light under a piece fill it up to the outline and stop crisply there, with any bloom as a separate halo outside? | 3x of a lit pill and the rev bead, `03_redline_strobe_crop.png` | a glow blob that ignores the lens edge |
| G9 | Does the middle of each piece mirror nothing (black over black), with reflection only towards the edge? | 3x across a piece, `02` and `04` crops | a Fresnel band painted a fixed distance in, or a sheen arc fixed in place (optics §3, §7) |
| G10 | Where a piece crosses the cockpit photo, does the photo show almost unchanged through the middle and drop away only in a thin band at the edge? | `02_mid_cruise_cockpit.png`, 1:1 of every piece over the photo; skip if all sit on the scrim core | the photo dimmed or tinted evenly under the whole piece (optics §8) |
| G11 | Over black, is every body colourless, and where a tint shows over lit content is it deepest through the thickest glass and almost absent on thin pills? | 3x of a thick dome and a thin pill, `03` over lit content and `01` over black | one tint film everywhere, including over black (optics §6) |
| G12 | Are rims over black free of paired red and blue fringes, and thin pieces free of colour separation at 1:1? | 3x of rims in `01_idle_crop.png` | a fixed chromatic split on every outline (optics §5) |
| G13 | Where separation does show (the thickest bevel over lit content), is it in the same order on every side, blue nearer the rim, wider where thicker? | 3x of the thickest rim at top, bottom, left and right, `03` crop | fringes whose order flips |
| G14 | For the Liquid Glass family, are all reflections free of violet, purple or rainbow sheen? | 3x of rims and key highlights, `02` crop | an anti-reflection tint on uncoated glass (optics §9) |
| G15 | Are the pieces separate floating shapes with true black between them at rest? | full frame and 1:1 of `01` and `02` | one plate with pills stuck on, or a grey floor between pieces |
| G16 | Where two pieces fuse, does each rim run smoothly round the neck with no kink, crease or seam? | 3x of each neck in `04_drift_braking_crop.png` and `03` | a V-notch, overlapping outlines, a bar pasted between pieces (optics §2) |
| G17 | Does the largest piece bend more (stronger magnification and rim stretch) than the small pills? | 1:1 of the largest dome and a small pill | the same treatment stamped on every size |
| G18 | Is every glass piece free of another glass piece stacked on it? | 1:1 of all four states | a pill on a glass panel, doubled rims, layered milk |
| G19 | In the cockpit composite, does every numeral seen through glass keep full contrast, with no rim highlight or caustic crossing its strokes? | `<state>_cockpit.png`, 1:1 of the gear and speed zones, plus `glance` | a streak across a digit, digits washed out, a black box outside the glass |
| G20 | Are capsule ends true half circles and nested shapes concentric with even insets? | 3x of capsule ends | pinched or flared corners, thickening rims ([WWDC25 356](https://developer.apple.com/videos/play/wwdc2025/356/)) |
| G21 | If a caustic is drawn, does it sit on the side away from the key light and stay opposite it as the light swings? | 1:1 of `02` and `04` side by side | a caustic image fixed in place |
| G22 | Where a crystal is coated, is its reflection faint (about a tenth of bare glass) and does its colour change from the flat centre to the curved edge? | 3x of a coated crystal | one flat purple overlay (optics §9) |
| G23 | Where a prism throws colour, is the order fixed (blue bent most) and does the spread change as the prism turns? | 3x of the prism in `02` and `04` | channels offset by a made-up amount (optics §10) |

## Light and shadow

| # | Check (yes/no) | Look at | The fake it catches |
|---|---|---|---|
| L1 | Between `02_mid_cruise` (0.18 g) and `04_drift_braking` (-0.96 g), have the key-side highlights on every piece moved by the same angle in the same direction? | 3x of two pieces in both states on one sheet; `diff` of the pair should light every rim | highlights that ignore the light, or move on some pieces only |
| L2 | Do all highlights (glass rims, metal bevels, LED caps, needle ridges, hubs) agree on one key light and one cool rim light opposite? | 1:1 of `02`, highlights marked with `annotate` | parts lit from different sides, or sprites rendered under another rig |
| L3 | In the drift state, does each highlight appear once, with no fainter copy beside it? | 3x of chamfers, hubs and crystals in `04` | double highlights from crossfading two light renders |
| L4 | As the needle sweeps (frames 10 to 29), does its lit side stay towards the key light? | 3x of the needle in frames 10, 20 and 29 | baked lighting that turns with the sprite |
| L5 | Do the rounded edges of non-metal parts catch a brighter reflection than their flat faces? | 3x of edges beside faces | even reflection everywhere ([everything has Fresnel](http://filmicworlds.com/blog/everything-has-fresnel/)) |
| S1 | Where one part sits on another (bezel on face, hub on dial), is there a thin dark contact line that fades away from the joint? | 3x of each joint | parts floating with no contact darkening, or an even black outline |
| S2 | Does each raised part cast its shadow away from the key light, further and softer the higher it stands? | 1:1 of bezels, crystals, hubs and housings | an even halo shadow, or a shadow towards the light ([shadow demos](https://kerstenlab.psych.umn.edu/demos/shadows)) |
| S3 | Does a live needle cast a soft shadow on the dial that moves with it? | 1:1 of the needle zone across frames 10 to 29 | a needle like a sticker |
| S4 | Are plate and sprite edges free of dark or pale fringes? | 3x of edges | alpha applied twice (a dark ring) or straight alpha drawn as premultiplied (a pale fringe) |

## Metal, carbon, LEDs and trims

| # | Check (yes/no) | Look at | The fake it catches |
|---|---|---|---|
| M1 | Does every metal part reflect like metal (bright beside dark, no flat diffuse grey), in its own colour? | 1:1 of each metal part | flat grey paint, faces black because they mirror nothing, a faint tint on a vector shape |
| M2 | On large faces, does the reflection break up a little with roughness variation and fine grain? | 1:1 of the largest metal face | a mathematically smooth gradient |
| M3 | On brushed, sunray and spun finishes, does the highlight run across the grain (a spun bezel shows a streak through its centre towards the light; a sunray dial a band across the light direction)? | 1:1 and 3x of each anisotropic part | highlights running along the grain ([brushed metal](https://www.keyshot.com/blog/how-to-render-brushed-metals-in-keyshot/)) |
| M4 | Between `02` and `04`, have the bands on sunray and spun parts and the chamfer highlights turned with the light? | 1:1 of both states on one sheet | a frozen plate while the glass highlights move |
| M5 | Does every machined edge carry a thin continuous highlight on the key side, with the opposite chamfer darker? | 1:1 and 3x of bezels and hubs | knife edges with no catch light, or equal brightness all round |
| M6 | Do relief patterns (knurling, guilloché, leather grain) read raised or cut as on the real part, each bump lit on its key side? | 1:1 and 3x of each relief | grey noise with no light direction |
| M7 | Do the brightest highlights have a hot core that rolls off smoothly? | 3x of the brightest highlights | flat #FFFFFF plateaus with hard edges |
| M8 | Do parts with the same finish show the same highlight size and sharpness everywhere? | 1:1 of parts sharing a finish | sharp and blurry chrome side by side |
| C1 | Does the carbon show a sharp clear-coat reflection floating over a weave whose crossing tows reflect differently? | 1:1 of the carbon in `02` and `04` | a printed checker (carbon-look vinyl) |
| C2 | Between `02` and `04`, does the tows' light and dark pattern change as the light turns? | 1:1 of both, or `diff` over the carbon | a static texture |
| C3 | Is the weave at a real fabric's scale (3K twill: about 2 mm a tow, about 6 px on this panel when drawn life size)? | 1:1, counting tows over a measured span | toy-sized checks or a weave that averages to grey |
| E1 | Does an unlit LED cap still read as a lens, with a small sharp highlight on the key side? | 3x of unlit caps in `01` | a flat black or grey dot |
| E2 | Does each lit cap light like its real lens (even for a diffused lens, a hot spot for a clear one), with the surface highlight still on top? | 3x of lit caps in `03` beside the reference photo | a flat coloured disc or rectangle of glow |
| E3 | Through the strobe (frames 30 to 41), do the caps stay visible as hardware in the off phase, with only the light inside switching? | 3x of the shift lights across frames 30 to 41 | whole caps blinking in and out |
| E4 | Does a crystal over a dial show only reflections, leaving the dial at full contrast? | 1:1 of each crystal-covered dial | a grey veil or a painted gloss arc |
| T1 | Does piano black read as near #000000 with a crisp reflection band? | 1:1 of piano-black trims | lifted dark-grey matte |
| T2 | Does leather show fine irregular grain at the hide's scale, a broad soft sheen, and stitches in shadowed holes, with no visible repeat? | 1:1 and 3x of the stitching | a repeating tile, orange-peel grain, a dashed line for stitches |
| T3 | Does alcantara read as soft matte suede with a gentle lift at grazing edges? | 1:1 of the trims | a plastic highlight or flat grey |

## Motion of light and glass

| # | Check (yes/no) | Look at | The fake it catches |
|---|---|---|---|
| N1 | Where the dash leans its light with acceleration, do highlights shift smoothly and together through frames 9 to 29 and 41 to 45? | frames 9, 15, 20, 29, 41, 43, 45 at 3x, or `motion`; skip if the light follows lateral G only (held at 0.12 g in the sequence) | highlights frozen through the pull or jumping in one frame |
| N2 | Are thin highlights on tight curves continuous at 3x and steady from frame to frame? | 3x of pill ends, the bead and a needle tip across frames 10 to 29 | specular aliasing: dots that crawl or flicker |
| N3 | During the upshift (frames 42 to 50), do the gear piece's rim reflections wobble while the numeral under its middle stays sharp? | frames 41 to 50 at 3x | the numeral warping with the wave, or a ring drawn on top |
| N4 | When pieces merge or split, do they reach through a growing neck while their centres stay put? | frames 29 to 42 at 1:1, and `02` against `04` | pieces sliding together, crossfades, pops (optics §2) |
| N5 | Does a rim next to a lit light mirror it as a small streak that follows it on and off through the strobe? | 3x of the rim nearest the shift lights, `02` against `03` and frames 30 to 41 | rims that ignore the lights beside them |

## Whole-part checks

| # | Check (yes/no) | Look at | The fake it catches |
|---|---|---|---|
| W1 | Without zooming, in the full frame and the cockpit composite, can you name each part's material? | each full frame (read at about 0.41x) and each `<state>_cockpit.png` | materials that only appear at 2x or 3x |
| W2 | Beside the board export and the before render, does the new render look clearly richer (materials, light, depth)? | one sheet: board, before, new | render and board look alike |
| W3 | Does each material match the finish of the real part in the reference photos (gloss level, metal or paint, unlit lens colour, weave)? | a sheet of each reference beside the matching 1:1 crop | generic chrome or carbon on parts that are matte or painted on the real car |
