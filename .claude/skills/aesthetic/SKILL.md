---
name: aesthetic
description: Create aesthetically beautiful interfaces following proven design principles. Use when building UI/UX, analyzing designs from inspiration sites, implementing visual hierarchy and color theory, adding micro-interactions, or creating design documentation. Includes workflows for capturing and analyzing inspiration screenshots with the built-in browser and for reviewing generated design images against a fixed yes/no checklist, and comprehensive design system guidance covering BEAUTIFUL (aesthetic principles), RIGHT (functionality/accessibility), SATISFYING (micro-interactions), and PEAK (storytelling) stages.
---

# Aesthetic

Create aesthetically beautiful interfaces by following proven design principles and systematic workflows.

## When to Use This Skill

Use when:
- Building or designing user interfaces
- Analyzing designs from inspiration websites (Dribbble, Mobbin, Behance)
- Generating design images and evaluating aesthetic quality
- Implementing visual hierarchy, typography, color theory
- Adding micro-interactions and animations
- Creating design documentation and style guides
- Need guidance on accessibility and design systems

## Core Framework: Four-Stage Approach

### 1. BEAUTIFUL: Understanding Aesthetics
Study existing designs, identify patterns, extract principles. AI lacks aesthetic sense—standards must come from analyzing high-quality examples and aligning with market tastes.

**Reference**: [`references/design-principles.md`](references/design-principles.md) - Visual hierarchy, typography, color theory, white space principles.

### 2. RIGHT: Ensuring Functionality
Beautiful designs lacking usability are worthless. Study design systems, component architecture, accessibility requirements.

**Reference**: [`references/design-principles.md`](references/design-principles.md) - Design systems, component libraries, WCAG accessibility standards.

### 3. SATISFYING: Micro-Interactions
Incorporate subtle animations with appropriate timing (150-300ms), easing curves (ease-out for entry, ease-in for exit), sequential delays.

**Reference**: [`references/micro-interactions.md`](references/micro-interactions.md) - Duration guidelines, easing curves, performance optimization.

### 4. PEAK: Storytelling Through Design
Elevate with narrative elements—parallax effects, particle systems, thematic consistency. Use restraint: "too much of anything isn't good."

**Reference**: [`references/storytelling-design.md`](references/storytelling-design.md) - Narrative elements, scroll-based storytelling, interactive techniques.

## Workflows

### Workflow 1: Capture & Analyze Inspiration

**Purpose**: Extract design guidelines from inspiration websites.

**Steps**:
1. Browse inspiration sites (Dribbble, Mobbin, Behance, Awwwards)
2. Use the built-in browser to capture full-screen screenshots (not full page)
3. Analyze the screenshots yourself and extract:
   - Design style (Minimalism, Glassmorphism, Neo-brutalism, etc.)
   - Layout structure & grid systems
   - Typography system & hierarchy
     **IMPORTANT:** Try to predict the font name (Google Fonts) and font size in the given screenshot, don't just use Inter or Poppins.
   - Color palette with hex codes
   - Visual hierarchy techniques
   - Component patterns & styling
   - Micro-interactions
   - Accessibility considerations
   - Overall aesthetic quality, described in words (no numeric rating)
4. Document findings in project design guidelines using templates

### Workflow 2: Generate & Iterate Design Images

**Purpose**: Create aesthetically pleasing design images through iteration.

**Steps**:
1. Define design prompt with: style, colors, typography, audience, animation specs
2. Generate design images with ComfyUI (the `comfyui-hero-textures` skill), holding the GPU lock
3. Review the output against the project's fixed checklist (in this repo: the `fh6-visual-review` skill for renders, boards and motion, and "Closing review" in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`). Answer each line yes or no; never give a numeric score
4. For each no:
   - Identify the specific weakness (color, typography, layout, spacing, hierarchy)
   - Fix that weakness, then walk the checklist again
5. In this repo the thread signs off its own result with that checklist, because TJX is hands-off (AGENTS.md rule 12); TJX's own words win whenever TJX answers. Never loop until a score passes, and never rewrite a prompt to chase a score
6. Document final design decisions using templates

## Design Documentation

### Create Design Guidelines
Use [`assets/design-guideline-template.md`](assets/design-guideline-template.md) to document:
- Color patterns & psychology
- Typography system & hierarchy
- Layout principles & spacing
- Component styling standards
- Accessibility considerations
- Design highlights & rationale

Save in project `./docs/design-guideline.md`.

### Create Design Story
Use [`assets/design-story-template.md`](assets/design-story-template.md) to document:
- Narrative elements & themes
- Emotional journey
- User journey & peak moments
- Design decision rationale

Save in project `./docs/design-story.md`.

## Resources & Integration

### Related Skills
- **comfyui-hero-textures**: Generated textures and images through ComfyUI
- **The built-in browser**: Screenshots of inspiration sites and of the rendered pages
- **frontend-design**: Web UI direction and typography
- **fh6-hmi-design-rules**: The project's own design rules, which win where the two differ
- **fh6-visual-review**: The project's fixed visual checklist and the `visual-review` tool, for reviewing renders without scores

### Reference Documentation
**References**: [`references/design-resources.md`](references/design-resources.md) - Inspiration platforms, design systems, AI tools, MCP integrations, development strategies.

## Key Principles

1. Aesthetic standards come from humans, not AI—study quality examples
2. Iterate based on analysis—never settle for first output
3. Balance beauty with functionality and accessibility
4. Document decisions for consistency across development
5. Use progressive disclosure in design—reveal complexity gradually
6. Always evaluate against the fixed checklist, with yes or no answers and no numeric scores; in this repo the thread's own review signs off (AGENTS.md rule 12)
