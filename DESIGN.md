---
name: exhume
description: One molded surface that the dead code rises out of.
colors:
  slate-clay: "#d9dde0"
  slate-clay-highlight: "#f8fafb"
  slate-clay-shadow: "#aeb5bb"
  slate-clay-rule: "#c9cfd4"
  graphite-ink: "#1b2024"
  graphite-mute: "#464f57"
  verdigris: "#0d6152"
  verdigris-select: "#b9dcd4"
  night-clay: "#242a2f"
  night-clay-highlight: "#2f373d"
  night-clay-shadow: "#161a1d"
  night-clay-rule: "#1d2226"
  frost-ink: "#e7ecef"
  frost-mute: "#a5b0b8"
  verdigris-lit: "#46c9ad"
  verdigris-lit-select: "#2c5c52"
typography:
  display:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "clamp(42px, 4.8vw, 68px)"
    fontWeight: 800
    lineHeight: 0.98
    letterSpacing: "-0.04em"
  headline:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "clamp(34px, 4.2vw, 56px)"
    fontWeight: 800
    lineHeight: 1
    letterSpacing: "-0.035em"
  title:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 700
    lineHeight: 1.3
    letterSpacing: "-0.02em"
  body:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "18px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
  label-quiet:
    fontFamily: "Schibsted Grotesk, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 500
  code:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.55
    fontFeature: "\"tnum\", \"liga\" 0, \"calt\" 0"
  exit-digit:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: "34px"
    fontWeight: 600
    lineHeight: 1
    fontFeature: "\"tnum\""
rounded:
  chip: "12px"
  tile: "14px"
  control: "16px"
  keycap: "20px"
  panel: "26px"
  slab: "36px"
  ground: "44px"
spacing:
  xs: "8px"
  sm: "14px"
  md: "22px"
  lg: "32px"
  xl: "48px"
  section: "clamp(80px, 12vh, 140px)"
  gutter: "clamp(16px, 5vw, 72px)"
components:
  button-raised:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.verdigris}"
    typography: "{typography.label}"
    rounded: "{rounded.control}"
    padding: "14px 20px"
  button-raised-hover:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
  button-raised-dark:
    backgroundColor: "{colors.night-clay}"
    textColor: "{colors.verdigris-lit}"
  button-raised-small:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.tile}"
    padding: "10px 18px"
  install-slab:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-mute}"
    typography: "{typography.label-quiet}"
    rounded: "{rounded.panel}"
    padding: "22px 22px 22px 26px"
  command-well:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.code}"
    rounded: "{rounded.tile}"
    padding: "14px 16px"
  file-tile:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-mute}"
    typography: "{typography.code}"
    rounded: "{rounded.tile}"
    padding: "12px 16px"
  file-tile-finding:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
  chip:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.label-quiet}"
    rounded: "{rounded.chip}"
    padding: "10px 16px"
  keycap:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.exit-digit}"
    rounded: "{rounded.keycap}"
    size: "72px"
  keycap-verdict:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    typography: "{typography.code}"
    rounded: "{rounded.tile}"
    padding: "10px 18px"
  exit-row:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    rounded: "{rounded.panel}"
    padding: "22px 26px"
  theme-switch:
    backgroundColor: "{colors.slate-clay}"
    textColor: "{colors.graphite-ink}"
    rounded: "{rounded.keycap}"
    width: "64px"
    height: "34px"
---

<!-- Scanned from the shipped build in exhume-landing/app (globals.css tokens, page.tsx, layout.tsx, components/*.tsx) on 2026-10-31. Supersedes the record taken from the pre-build mock .impeccable/mocks/design/landing.html; where the two differ, this file follows the build. -->

# Design System: exhume

## Overview

**Creative North Star: "The Molded Ground"**

The whole interface is one continuous soft material, pressed and raised like a single cast surface. Nothing sits on top of the page; everything is formed out of it. Ordinary content is pressed into the ground, and the things that matter, the findings and the actions, rise out of it. This matches what exhume does: it digs a workspace and lifts out the files nothing imports.

The material is neumorphic, but it serves a working tool, not an app-store showcase. Type is tight and heavy, paths and output are set in real monospace, and depth carries meaning rather than decoration. The surface exists in two full renditions, a cool daylight clay and a deep night clay. Neither is a filtered copy of the other.

Density is generous around headings and tight inside groups. One accent colour does all the signalling, and every line of output shown is real.

**Key Characteristics:**
- One material per theme; surfaces are pressed or raised, never layered or floating.
- Raised means "this matters": findings, primary actions, keycaps.
- Pressed means "this is context": wells, used files, chips, code.
- A single accent, Verdigris, marks confidence labels, the primary action and focus.
- Light and dark are both first-class and follow the OS, with a remembered manual switch.

## Colors

A cool, low-chroma clay in two lights, with one patina-green accent that does all the signalling. Each theme is the same eight roles (ground, highlight, shadow, rule, ink, mute, accent, select) swapped wholesale on `html[data-theme]`; the browser's own theme colour is set to the ground of each scheme.

### Primary
- **Verdigris** (light) / **Lit Verdigris** (dark): the only accent. Used for the `high` confidence label, the Copy key's label, the exit-code digit that means findings, the emphasised phrase in the display headline, and focus rings. In dark it brightens so contrast holds (5.4:1 and 7.1:1 on their grounds).

### Neutral
- **Slate Clay** / **Night Clay**: the ground. Every surface, button and tile is this colour; depth comes from the light and shadow around it, not from a different fill.
- **Clay Highlight** / **Night Highlight**: the lit edge, top-left of every raised or pressed form.
- **Clay Shadow** / **Night Shadow**: the shaded edge, bottom-right; also the page scrollbar thumb and the roadmap's dashed outline.
- **Clay Rule** / **Night Rule**: 2px divisions between strata, walk columns, roadmap rows and the footer.
- **Graphite Ink** / **Frost Ink**: headings, body emphasis, raised findings, button labels.
- **Graphite Mute** / **Frost Mute**: secondary copy, used files, prompts, nav links and notes. Measured at 6.1:1 and 6.6:1 on their grounds.
- **Verdigris Select** / **Lit Verdigris Select**: text selection only.

### Named Rules
**The One Patina Rule.** Verdigris is the only colour that is not clay or ink. It marks findings, the primary action and focus, and nothing decorative.

**The Same Clay Rule.** A control never gets its own fill. It is the ground colour, shaped by light. A filled button is a sticker on the material.

## Typography

**Display Font:** Schibsted Grotesk (with system-ui, sans-serif), loaded through next/font at 400 to 800
**Body Font:** Schibsted Grotesk
**Label/Mono Font:** JetBrains Mono (with ui-monospace, monospace), loaded at 400 and 600

**Character:** A dense, newsroom-grade grotesk at heavy weights against an honest coding mono. The grotesk sells; the mono only ever carries real paths, commands and output.

### Hierarchy
- **Display** (800, clamp(42px, 4.8vw, 68px), 0.98, -0.04em): the single page headline, two lines at most, max 16ch, balanced wrap. One phrase in Verdigris, set upright.
- **Headline** (800, clamp(34px, 4.2vw, 56px), 1.0, -0.035em): each stratum's heading, max 18ch, balanced wrap.
- **Title** (700, 22px, 1.3, -0.02em): sub-blocks inside a stratum and exit-row titles. The wordmark uses this size at 800.
- **Body** (400, 18px, 1.5): supporting copy in Mute; the lede holds 34ch, section intros 60ch, side notes 46ch.
- **Label** (700, 15px): button text. At 500 (Label Quiet) it sets nav links and workspace chips; at 400 it sets install notes, the ground header and the footer.
- **Code** (400 or 600, 15px, tabular numerals, ligatures off): commands, file paths, report output, package names. Line height 1.55 in wells, 1.6 in exit output, 1.7 in the CI block.
- **Exit Digit** (600, 34px mono): the digit inside an exit-code keycap, and nothing else.

### Named Rules
**The Real Mono Rule.** Monospace is for things a terminal would print: commands, paths, output. Never for decoration, eyebrows or labels. Workspace names in prose, such as the supported-setup chips, stay in the grotesk.

**The No Eyebrow Rule.** Headings carry their own weight. No kicker or small label above them.

**The Typed As Shown Rule.** Code renders exactly what a visitor would type: ligatures are disabled (`--` and `>-` stay two glyphs), and mono paths and commands break only at a space or right after a single `/`. `file.ts:1:1` stays whole, `pnpm-monorepo` never splits at its hyphen, and `https://` stays attached to its host.

## Layout

A single centred column, max 1280px, with fluid gutters (clamp(16px, 5vw, 72px)). The page reads as a stack of strata: each section is a horizontal band of the same ground, separated by large vertical space (clamp(80px, 12vh, 140px); the closing band clamp(96px, 14vh, 160px)) rather than by boxes.

The opening pairs the display headline with the install slab on a two-column grid (fluid text column, 480px slab column), aligned to the bottom, so the headline holds two lines beside the slab at 1440 wide. The closing band repeats the pairing with a 520px slab column, centred vertically. The ground beneath the opening spans the full column; its strata rows use a 220px package column beside a wrapping row of file tiles. Exit rows are a three-column grid: 96px keycap, up to 260px of title and meaning, then the output.

At 960px and below, every two-column grid stacks, the walk band becomes one column with horizontal rules, exit rows drop to keycap plus text with the output spanning beneath, and the roadmap list goes to one column. At 640px and below, strata stack package above files, the ground tightens to 26px radius, the copy key goes full width under the command, and nav keeps only GitHub and the theme switch.

No element may push the page wider than the viewport. Prose-like output wraps; column-aligned report output and the CI block keep their columns on phones and scroll sideways inside their well, with a right-edge mask fade (solid to 82%, then transparent) signalling more to the right.

## Elevation & Depth

Depth is the entire visual system. Every form is either raised (casts a shadow down-right, catches light up-left) or pressed (the same pair inverted, inset). There is no third state: no floating cards, no glow, no flat filled panels.

### Shadow Vocabulary
- **Raised** (`8px 8px 18px shadow, -8px -8px 18px highlight`): the install slab, the walk band, findings once surfaced.
- **Raised small** (`5px 5px 11px shadow, -5px -5px 11px highlight`): buttons, keycaps, the theme-switch knob.
- **Pressed** (`inset 5px 5px 10px shadow, inset -5px -5px 10px highlight`): command wells, used-file tiles, chips, the CI block, exit rows, the theme-switch track, and any button while pressed, copied or scanning.
- **Pressed deep** (`inset 10px 10px 22px shadow, inset -10px -10px 22px highlight`): the ground that holds the strata.

All four are declared with the same four-shadow structure (zero-length shadows fill the unused slots), so a form can travel from pressed to raised in one smooth transition.

### Named Rules
**The Rise Means Found Rule.** Only a finding or an action may be raised inside a pressed area. If it's raised, it must matter.

**The Never Shadow Alone Rule.** Raised versus pressed is never the only signal. A raised finding also changes ink weight and gains its `high` label, so the state survives low-contrast screens and colour-blind viewers.

## Shapes

Soft, consistent rounding that grows with the size of the form: chips (12px); tiles, wells, small keys and the verdict keycap (14px); buttons (16px); exit keycaps and the switch track (20px); the install slab, exit rows and the CI block (26px); the walk band and roadmap (36px); the ground itself (44px; 26px on phones). The switch knob is the one circle. Rules are flat 2px lines in the Rule colour. The roadmap is the one exception to solid form: a 2px dashed outline in Shadow, marking it as not yet cast.

## Components

### Buttons
Pressable and exact: physical keys that visibly sink, nothing bouncy.
- **Shape:** gently rounded key (16px), raised small, ground fill, 18px icon with a 2px stroke, 8px gap.
- **Primary (Copy):** Verdigris label. Hover shifts the label to Ink. Press flips it to pressed. After a click it stays pressed for 2 seconds as "Copied" with a check icon. The copy tries the clipboard API, then a hidden-textarea execCommand copy, and if both fail selects the command in its well and reads "Selected, copy it".
- **Secondary (Scan again):** the small key (14px radius, 10px 18px) with an Ink label and a replay icon. It sits pressed while a scan is running.
- **Focus:** 2px Verdigris outline at a 4px offset.
- **Transitions:** shadow and colour over 250ms on the ease-out curve.

### Theme Switch
A pressed 64 by 34px track holding a raised 26px round knob with a sun or moon icon. The knob slides 30px over 450ms on the ease-out curve. A pre-paint inline script applies the stored or OS theme to `html[data-theme]` before first paint, so there is no flash; the switch reads that attribute through an external store and writes the manual choice to local storage. Its accessible label names the theme it will switch to. Ground and ink crossfade over 400ms. Without JavaScript the switch is hidden and the clay follows the OS through `prefers-color-scheme`. A focused control keeps its own corner radius; the focus ring follows its shape.

### Chips
- **Style:** pressed, ground colour, Ink grotesk at 500 (12px radius, 10px 16px). Used for the list of supported workspace setups. Static, not interactive.

### Cards / Containers
There are no cards. Containers are wells and slabs of the same clay: the install slab (raised), the ground (pressed deep), exit rows (pressed), the CI block (pressed), the "how it reads" band (one raised slab divided by rules, never separate boxes).

### Command Well
A pressed mono field holding a real command, with a non-selectable `$ ` prompt in Mute. It sits in the raised install slab beside its copy key, between two Mute notes, and wraps only at the command's safe break points rather than scrolling sideways.

### File Tile (signature)
A mono tile for one file inside a stratum. A used file is pressed with Mute text. A finding is raised, in Ink at 600 weight, lifted 6px, with a Verdigris `high` label. When the ground scrolls a quarter into view the findings rise one by one in report order (500ms delay, 700ms rise, 420ms stagger), the label fading in 250ms behind its tile; 700ms after the last, the summary and exit keycap appear. With reduced motion they appear at once; without JavaScript they render already raised.

### Keycap
A raised square (72px, 20px radius) holding an exit code in the Exit Digit style. The digit is Verdigris only when it means findings (`1`). The verdict under the strata uses a smaller raised key (14px radius) reading `exit 1`, with the digit in Verdigris.

### Navigation
Wordmark at title size and 800 weight on the left; Mute Label Quiet links, then the theme switch, on the right. Links turn Ink on hover. On phones only GitHub and the switch remain.

## Do's and Don'ts

### Do:
- **Do** build every surface from the ground colour and shape it with the four-shadow vocabulary.
- **Do** keep body, path and button text at 4.5:1 or better against the ground, in both themes.
- **Do** render every state (copy default, pressed, copied, selected, the scan sequence, the reduced-motion and no-JavaScript fallbacks) in both themes.
- **Do** show real exhume output, unedited, wherever output appears.
- **Do** theme the browser surfaces: selection in Verdigris Select, scrollbars in Shadow on the ground, focus rings in Verdigris, the browser theme colour in the ground.
- **Do** let column-aligned output scroll inside its well on narrow screens, with the right-edge fade, rather than rewrap it.

### Don't:
- **Don't** dim or partially fade an element to show a state; neumorphism already runs low contrast. Opacity only takes an element from absent to present during the scan sequence.
- **Don't** give a button or panel its own fill colour, gradient, glow or glass.
- **Don't** nest a raised card inside a raised card, or float anything above the ground.
- **Don't** use monospace for anything that isn't a command, path or output.
- **Don't** let a mono path or command break mid-token or render a ligature.
- **Don't** set a sentence-long headline at display size beyond two lines.
- **Don't** let a soft, toy-like or app-store look creep in: no bouncy easing, pastel accents or rounded-blob illustrations.
