# Design Resources & Tools

## Inspiration Platforms

### Design Galleries
- **Dribbble**: High-quality UI/UX designs, trending styles
- **Mobbin**: Mobile app design patterns, real-world examples
- **Behance**: Creative portfolios, comprehensive projects
- **Awwwards**: Award-winning web experiences
- **21st.dev**: Component animations, micro-interactions
- **CSS Design Awards**: Cutting-edge web designs

### Design Systems
Search pattern: "design system + [style name]"
- Material Design (Google)
- Human Interface Guidelines (Apple)
- Carbon Design System (IBM)
- Ant Design
- Shadcn UI
- Atlassian Design System

## AI Tools for Design

### Generation
- **ComfyUI (SDXL)**: Image generation through the `comfyui-hero-textures` skill
- **Claude**: Design variations, component code

### Prompt Structure
Include in prompts:
- Task description
- Preferred design style
- Color palette
- Typography preferences
- Target audience/user story
- Animation specifications

## MCP Integrations

### Available MCPs
- **The built-in browser**: Research designs, analyze trends from Dribbble/Mobbin
- **Penpot MCP**: Read and write the design file (`docs/design/PENPOT.md`)

### Usage
Connect MCPs for enhanced capabilities when researching, analyzing, or implementing designs.

## Development Approach

### Git Worktrees Strategy
Maintain separate branches for different design style variations during development.
Allows parallel exploration of Minimalist, Modern, Glassmorphism interpretations.

### Parallel Agents
Use multiple agents to generate design style variations simultaneously, compare approaches.

## Documentation Standards

### Required Files
Create in `./docs/` directory:
- **design-guideline.md**: Color patterns, typography, layout principles, component styling, design highlights
- **design-story.md**: Narrative elements, thematic decisions, user journey considerations

### Content
Document:
- Rationale for design decisions
- Style guide (colors, fonts, spacing)
- Component patterns & usage
- Responsive breakpoints
- Accessibility considerations
- Brand alignment

### Maintenance
Update during development. Use development rules to remind Claude Code to follow guidelines consistently.
