use crate::core::cmd::EngineEvent;
use crate::core::render::graph::LogicalId;

use super::events::UiEvent;
use super::tree::{UiEventKind, UiTreeState};
use super::types::UiValue;

#[derive(Debug, Clone, Copy)]
pub enum UiAnimationProperty {
    Opacity,
    TranslateY,
}

#[derive(Debug, Clone, Copy)]
pub enum UiAnimationEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone)]
pub struct UiAnimation {
    pub node_id: LogicalId,
    pub property: UiAnimationProperty,
    pub from: f32,
    pub to: f32,
    pub duration_ms: u32,
    pub delay_ms: u32,
    pub easing: UiAnimationEasing,
    pub start_time: Option<f64>,
    pub completed: bool,
}

impl UiAnimation {
    pub fn value_at(&self, time: f64) -> f32 {
        let duration = (self.duration_ms as f64 / 1000.0).max(0.0001);
        let start_time = self.start_time.unwrap_or(time);
        let t = ((time - start_time) / duration).clamp(0.0, 1.0) as f32;
        let eased = apply_easing(self.easing, t);
        self.from + (self.to - self.from) * eased
    }
}

pub fn update_animations(
    animations: &mut Vec<UiAnimation>,
    overrides: &mut std::collections::HashMap<LogicalId, super::tree::UiStyle>,
    tree: &UiTreeState,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    now_seconds: f64,
) {
    overrides.clear();

    for animation in animations.iter_mut() {
        if animation.start_time.is_none() {
            animation.start_time =
                Some(now_seconds + animation.delay_ms as f64 / 1000.0_f64);
        }
        let start_time = animation.start_time.unwrap_or(now_seconds);
        if now_seconds < start_time {
            write_override(overrides, animation, animation.from);
            continue;
        }

        let elapsed = now_seconds - start_time;
        if elapsed >= animation.duration_ms as f64 / 1000.0_f64 {
            animation.completed = true;
            write_override(overrides, animation, animation.to);
        } else {
            let value = animation.value_at(now_seconds);
            write_override(overrides, animation, value);
        }
    }

    if animations.iter().any(|anim| anim.completed) {
        let mut remaining = Vec::with_capacity(animations.len());
        for animation in animations.drain(..) {
            if animation.completed {
                emit_anim_complete(
                    tree,
                    event_queue,
                    context_id,
                    window_id,
                    &animation.node_id,
                );
            } else {
                remaining.push(animation);
            }
        }
        *animations = remaining;
    }
}

fn write_override(
    overrides: &mut std::collections::HashMap<LogicalId, super::tree::UiStyle>,
    animation: &UiAnimation,
    value: f32,
) {
    let key = match animation.property {
        UiAnimationProperty::Opacity => "opacity",
        UiAnimationProperty::TranslateY => "translateY",
    };
    let entry = overrides
        .entry(animation.node_id.clone())
        .or_insert_with(std::collections::HashMap::new);
    entry.insert(key.to_string(), UiValue::Float(value as f64));
}

fn emit_anim_complete(
    tree: &UiTreeState,
    event_queue: &mut Vec<EngineEvent>,
    context_id: &LogicalId,
    window_id: u32,
    node_id: &LogicalId,
) {
    let Some(node) = tree.nodes.get(node_id) else {
        return;
    };
    let Some(listeners) = node.listeners.as_ref() else {
        return;
    };
    let Some(label) = listeners.on_anim_complete.clone() else {
        return;
    };
    event_queue.push(EngineEvent::Ui(UiEvent {
        window_id: LogicalId::Int(window_id as i64),
        context_id: context_id.clone(),
        label,
        kind: UiEventKind::AnimComplete,
        node_id: Some(node_id.clone()),
        value: None,
    }));
}

fn apply_easing(easing: UiAnimationEasing, t: f32) -> f32 {
    match easing {
        UiAnimationEasing::Linear => t,
        UiAnimationEasing::EaseIn => t * t,
        UiAnimationEasing::EaseOut => 1.0 - (1.0 - t).powi(2),
        UiAnimationEasing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
    }
}

pub fn parse_animation_property(value: &str) -> Option<UiAnimationProperty> {
    match value {
        "opacity" => Some(UiAnimationProperty::Opacity),
        "translateY" | "translate-y" => Some(UiAnimationProperty::TranslateY),
        _ => None,
    }
}

pub fn parse_animation_easing(value: Option<&str>) -> UiAnimationEasing {
    match value {
        Some("ease-in") => UiAnimationEasing::EaseIn,
        Some("ease-out") => UiAnimationEasing::EaseOut,
        Some("ease-in-out") => UiAnimationEasing::EaseInOut,
        _ => UiAnimationEasing::Linear,
    }
}
