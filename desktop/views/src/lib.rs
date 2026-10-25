use std::path::{Path, PathBuf};

use fm_core::{MermaidDiagramIr, MermaidRenderMode, MermaidTier};
use fm_layout::{CycleStrategy, DiagramLayout, EdgeRouting, LayoutAlgorithm, LayoutConfig, LayoutGuardrails, layout_diagram_traced_with_config_and_guardrails};
use fm_render_svg::{A11yConfig, SvgRenderConfig, render_svg_with_layout};
use fm_render_term::{TermRenderConfig, render_term_with_layout_and_config};

const TERM_COLUMNS: usize = 110;
const TERM_ROWS: usize = 60;
const PAGE: &str = "# Push-to-talk state machine\n\nGenerated from `step` in [`src/lib.rs`](src/lib.rs) by `cargo run -q` in `desktop/views/`, drawn with [frankenmermaid](https://github.com/Dicklesworthstone/frankenmermaid). States are rectangles; each transition is a node of its own (a hexagon when it taps Space), so the layout keeps every label clear; dashed edges are the ones the layout reversed to break cycles. Self-loops that change nothing are left out. The SVG is rendered with frankenmermaid's raster-safe settings (no CSS custom properties), so any SVG viewer draws it, not only browsers. `cargo test` in `desktop/views/` fails while these files differ from `step`; `cargo kani` in `talk/` proves the properties in `src/proofs.rs`.\n\n![push-to-talk state machine](machine.svg)\n\nSource: [`machine.mmd`](machine.mmd). Terminal rendering: [`machine.txt`](machine.txt).\n";

pub struct View {
    pub path: PathBuf,
    pub text: String,
}

pub struct Rendered {
    pub views: Vec<View>,
    pub warnings: Vec<String>,
}

fn portable_svg() -> SvgRenderConfig {
    let base: SvgRenderConfig = SvgRenderConfig::default();
    SvgRenderConfig {
        responsive: false,
        embed_theme_css: false,
        animations_enabled: false,
        print_optimized: false,
        shadows: false,
        glow_enabled: false,
        a11y: A11yConfig { accessibility_css: false, ..base.a11y.clone() },
        ..base
    }
}

pub fn talk_machine(talk_dir: &Path) -> Rendered {
    let source: String = vb_talk::mermaid();
    let parsed: fm_parser::ParseResult = fm_parser::parse(&source);
    let ir: &MermaidDiagramIr = &parsed.ir;
    let config: LayoutConfig = LayoutConfig { cycle_strategy: CycleStrategy::DfsBack, edge_routing: EdgeRouting::Spline, ..LayoutConfig::default() };
    let layout: DiagramLayout = (*layout_diagram_traced_with_config_and_guardrails(ir, LayoutAlgorithm::Sugiyama, config, LayoutGuardrails::default()).layout).clone();
    let svg: String = render_svg_with_layout(ir, &layout, &portable_svg());
    let term_config: TermRenderConfig = TermRenderConfig { render_mode: MermaidRenderMode::HalfBlock, tier: MermaidTier::Rich, ..TermRenderConfig::default() };
    let term: String = render_term_with_layout_and_config(ir, &layout, &term_config, TERM_COLUMNS, TERM_ROWS).output;
    let page: String = String::from(PAGE);
    Rendered {
        views: vec![
            View { path: talk_dir.join("machine.mmd"), text: source },
            View { path: talk_dir.join("machine.svg"), text: svg },
            View { path: talk_dir.join("machine.txt"), text: term },
            View { path: talk_dir.join("machine.md"), text: page },
        ],
        warnings: parsed.warnings,
    }
}
