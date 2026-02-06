pub mod common;
pub mod demo_001;
pub mod demo_002;
pub mod demo_003;
pub mod demo_004;
mod demo_004_graph;
pub mod demo_005;
pub mod demo_006;
pub mod demo_007;
mod demo_007_ui;
pub mod demo_008;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    Demo001,
    Demo002,
    Demo003,
    Demo004,
    Demo005,
    Demo006,
    Demo007,
    Demo008,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "demo_001" | "demo001" | "1" => Some(Self::Demo001),
            "demo_002" | "demo002" | "2" => Some(Self::Demo002),
            "demo_003" | "demo003" | "3" => Some(Self::Demo003),
            "demo_004" | "demo004" | "4" => Some(Self::Demo004),
            "demo_005" | "demo005" | "5" => Some(Self::Demo005),
            "demo_006" | "demo006" | "6" => Some(Self::Demo006),
            "demo_007" | "demo007" | "7" => Some(Self::Demo007),
            "demo_008" | "demo008" | "8" => Some(Self::Demo008),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Demo001 => "Vulfram Demo 001",
            Self::Demo002 => "Vulfram Demo 002",
            Self::Demo003 => "Vulfram Demo 003",
            Self::Demo004 => "Vulfram Demo 004",
            Self::Demo005 => "Vulfram Demo 005 (UI)",
            Self::Demo006 => "Vulfram Demo 006 (UI Screen)",
            Self::Demo007 => "Vulfram Demo 007 (Viewport)",
            Self::Demo008 => "Vulfram Demo 008 (UI + Render Integration)",
        }
    }
}

pub fn select_demo() -> DemoKind {
    if let Some(arg) = std::env::args().nth(1) {
        if let Some(demo) = DemoKind::from_str(&arg) {
            println!("Selected demo from args: {:?}", demo);
            return demo;
        }
    }

    if let Ok(value) = std::env::var("VULFRAM_DEMO") {
        if let Some(demo) = DemoKind::from_str(&value) {
            println!("Selected demo from env: {:?}", demo);
            return demo;
        }
    }

    DemoKind::Demo001
}

pub fn run_demo(demo: DemoKind, window_id: u32) -> bool {
    match demo {
        DemoKind::Demo001 => demo_001::run(window_id),
        DemoKind::Demo002 => demo_002::run(window_id),
        DemoKind::Demo003 => demo_003::run(window_id),
        DemoKind::Demo004 => demo_004::run(window_id),
        DemoKind::Demo005 => demo_005::run(window_id),
        DemoKind::Demo006 => demo_006::run(window_id),
        DemoKind::Demo007 => demo_007::run(window_id),
        DemoKind::Demo008 => demo_008::run(window_id),
    }
}
