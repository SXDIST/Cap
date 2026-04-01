use cap_project::{ClickSpringConfig, CursorEvents, ScreenMovementSpring, XY, ZoomSegment};

use crate::{
    Coord, RawDisplayUVSpace,
    cursor_interpolation::PrecomputedCursorTimeline,
    spring_mass_damper::SpringMassDamperSimulationConfig,
};

#[derive(Clone, Copy)]
struct CameraShiftTween {
    from: XY<f32>,
    to: XY<f32>,
    start_time_ms: f64,
    end_time_ms: f64,
}

struct ZoomFocusPrecomputeSim {
    position: XY<f32>,
    active_tween: Option<CameraShiftTween>,
    last_integrated_ms: f64,
}

const SAMPLE_INTERVAL_MS: f64 = 8.0;
const CLUSTER_WIDTH_RATIO: f64 = 0.5;
const CLUSTER_HEIGHT_RATIO: f64 = 0.7;

#[derive(Clone)]
struct SmoothedFocusEvent {
    time: f64,
    position: XY<f32>,
}

struct ClickCluster {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    start_time_ms: f64,
}

impl ClickCluster {
    fn new(x: f64, y: f64, time_ms: f64) -> Self {
        Self {
            min_x: x,
            max_x: x,
            min_y: y,
            max_y: y,
            start_time_ms: time_ms,
        }
    }

    fn can_add(&self, x: f64, y: f64, max_w: f64, max_h: f64) -> bool {
        let new_w = self.max_x.max(x) - self.min_x.min(x);
        let new_h = self.max_y.max(y) - self.min_y.min(y);
        new_w <= max_w && new_h <= max_h
    }

    fn add(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }
}

fn build_clusters(
    cursor_events: &CursorEvents,
    segment_start_secs: f64,
    segment_end_secs: f64,
    zoom_amount: f64,
) -> Vec<ClickCluster> {
    let start_ms = segment_start_secs * 1000.0;
    let end_ms = segment_end_secs * 1000.0;
    let cluster_w = CLUSTER_WIDTH_RATIO / zoom_amount;
    let cluster_h = CLUSTER_HEIGHT_RATIO / zoom_amount;

    let events_in_range: Vec<&cap_project::CursorMoveEvent> = cursor_events
        .moves
        .iter()
        .filter(|m| m.time_ms >= start_ms && m.time_ms <= end_ms)
        .collect();

    if events_in_range.is_empty() {
        let fallback = cursor_events
            .moves
            .iter()
            .rev()
            .find(|m| m.time_ms <= start_ms)
            .or_else(|| cursor_events.moves.iter().find(|m| m.time_ms >= start_ms));

        if let Some(evt) = fallback {
            return vec![ClickCluster::new(evt.x, evt.y, evt.time_ms)];
        }
        return vec![];
    }

    let mut clusters = Vec::new();
    let first = events_in_range[0];
    let mut current = ClickCluster::new(first.x, first.y, first.time_ms);

    for evt in &events_in_range[1..] {
        if current.can_add(evt.x, evt.y, cluster_w, cluster_h) {
            current.add(evt.x, evt.y);
        } else {
            clusters.push(current);
            current = ClickCluster::new(evt.x, evt.y, evt.time_ms);
        }
    }
    clusters.push(current);

    clusters
}

fn cluster_center_at_time(clusters: &[ClickCluster], time_ms: f64) -> Option<(f64, f64)> {
    clusters
        .iter()
        .rev()
        .find(|c| c.start_time_ms <= time_ms)
        .or_else(|| clusters.first())
        .map(|c| c.center())
}

struct SegmentClusters {
    start_secs: f64,
    end_secs: f64,
    zoom_amount: f64,
    edge_snap_ratio: f64,
    clusters: Vec<ClickCluster>,
}

pub struct ZoomFocusInterpolator {
    events: Option<Vec<SmoothedFocusEvent>>,
    precompute_sim: Option<ZoomFocusPrecomputeSim>,
    cursor_events: std::sync::Arc<CursorEvents>,
    _precomputed_cursor: Option<std::sync::Arc<PrecomputedCursorTimeline>>,
    _cursor_smoothing: Option<SpringMassDamperSimulationConfig>,
    _click_spring: ClickSpringConfig,
    screen_spring: ScreenMovementSpring,
    duration_secs: f64,
    segment_clusters: Vec<SegmentClusters>,
}

impl ZoomFocusInterpolator {
    pub fn new(
        cursor_events: &CursorEvents,
        cursor_smoothing: Option<SpringMassDamperSimulationConfig>,
        click_spring: ClickSpringConfig,
        screen_spring: ScreenMovementSpring,
        duration_secs: f64,
        zoom_segments: &[ZoomSegment],
    ) -> Self {
        Self::new_with_precomputed_cursor(
            cursor_events,
            cursor_smoothing,
            click_spring,
            screen_spring,
            duration_secs,
            zoom_segments,
            None,
        )
    }

    pub fn new_with_precomputed_cursor(
        cursor_events: &CursorEvents,
        cursor_smoothing: Option<SpringMassDamperSimulationConfig>,
        click_spring: ClickSpringConfig,
        screen_spring: ScreenMovementSpring,
        duration_secs: f64,
        zoom_segments: &[ZoomSegment],
        precomputed_cursor: Option<std::sync::Arc<PrecomputedCursorTimeline>>,
    ) -> Self {
        let segment_clusters = Self::build_segment_clusters(cursor_events, zoom_segments);
        Self {
            events: None,
            precompute_sim: None,
            cursor_events: std::sync::Arc::new(cursor_events.clone()),
            _precomputed_cursor: precomputed_cursor,
            _cursor_smoothing: cursor_smoothing,
            _click_spring: click_spring,
            screen_spring,
            duration_secs,
            segment_clusters,
        }
    }

    pub fn new_arc(
        cursor_events: std::sync::Arc<CursorEvents>,
        cursor_smoothing: Option<SpringMassDamperSimulationConfig>,
        click_spring: ClickSpringConfig,
        screen_spring: ScreenMovementSpring,
        duration_secs: f64,
        zoom_segments: &[ZoomSegment],
    ) -> Self {
        Self::new_arc_with_precomputed_cursor(
            cursor_events,
            cursor_smoothing,
            click_spring,
            screen_spring,
            duration_secs,
            zoom_segments,
            None,
        )
    }

    pub fn new_arc_with_precomputed_cursor(
        cursor_events: std::sync::Arc<CursorEvents>,
        cursor_smoothing: Option<SpringMassDamperSimulationConfig>,
        click_spring: ClickSpringConfig,
        screen_spring: ScreenMovementSpring,
        duration_secs: f64,
        zoom_segments: &[ZoomSegment],
        precomputed_cursor: Option<std::sync::Arc<PrecomputedCursorTimeline>>,
    ) -> Self {
        let segment_clusters = Self::build_segment_clusters(cursor_events.as_ref(), zoom_segments);
        Self {
            events: None,
            precompute_sim: None,
            cursor_events,
            _precomputed_cursor: precomputed_cursor,
            _cursor_smoothing: cursor_smoothing,
            _click_spring: click_spring,
            screen_spring,
            duration_secs,
            segment_clusters,
        }
    }

    fn build_segment_clusters(
        cursor_events: &CursorEvents,
        zoom_segments: &[ZoomSegment],
    ) -> Vec<SegmentClusters> {
        zoom_segments
            .iter()
            .filter(|s| matches!(s.mode, cap_project::ZoomMode::Auto))
            .map(|s| SegmentClusters {
                start_secs: s.start,
                end_secs: s.end,
                zoom_amount: s.amount,
                edge_snap_ratio: s.edge_snap_ratio,
                clusters: build_clusters(cursor_events, s.start, s.end, s.amount),
            })
            .collect()
    }

    fn zoom_amount_at(&self, time_secs: f64) -> f64 {
        self.segment_clusters
            .iter()
            .find(|sc| time_secs >= sc.start_secs && time_secs <= sc.end_secs)
            .map(|sc| sc.zoom_amount)
            .unwrap_or(1.0)
    }

    fn cluster_focus_at(&self, time_secs: f64) -> Option<(f64, f64)> {
        let time_ms = time_secs * 1000.0;
        self.segment_clusters
            .iter()
            .find(|sc| time_secs >= sc.start_secs && time_secs <= sc.end_secs)
            .and_then(|sc| cluster_center_at_time(&sc.clusters, time_ms))
    }

    fn raw_focus_target_at(&self, time_secs: f32) -> XY<f32> {
        if let Some(pos) = Self::raw_cursor_position_at(&self.cursor_events, time_secs) {
            return pos;
        }

        if let Some((cx, cy)) = self.cluster_focus_at(time_secs as f64) {
            return XY::new(cx.clamp(0.0, 1.0) as f32, cy.clamp(0.0, 1.0) as f32);
        }

        XY::new(0.5, 0.5)
    }

    fn active_segment_at(&self, time_secs: f64) -> Option<&SegmentClusters> {
        self.segment_clusters
            .iter()
            .find(|sc| time_secs >= sc.start_secs && time_secs <= sc.end_secs)
    }

    fn edge_shift_target(
        current_center: XY<f32>,
        cursor_pos: XY<f32>,
        zoom_amount: f64,
        edge_snap_ratio: f64,
    ) -> XY<f32> {
        let zoom = zoom_amount.max(1.0) as f32;
        let viewport_half = 0.5 / zoom;
        let viewport_size = viewport_half * 2.0;
        let margin = (viewport_size * edge_snap_ratio.clamp(0.08, 0.35) as f32)
            .clamp(viewport_size * 0.08, viewport_size * 0.33);

        let mut left = current_center.x - viewport_half;
        let mut top = current_center.y - viewport_half;
        let right = current_center.x + viewport_half;
        let bottom = current_center.y + viewport_half;

        let inner_left = left + margin;
        let inner_right = right - margin;
        let inner_top = top + margin;
        let inner_bottom = bottom - margin;

        if cursor_pos.x < inner_left {
            left = cursor_pos.x - margin;
        } else if cursor_pos.x > inner_right {
            left = cursor_pos.x + margin - viewport_size;
        }

        if cursor_pos.y < inner_top {
            top = cursor_pos.y - margin;
        } else if cursor_pos.y > inner_bottom {
            top = cursor_pos.y + margin - viewport_size;
        }

        let max_origin = (1.0 - viewport_size).max(0.0);
        left = left.clamp(0.0, max_origin);
        top = top.clamp(0.0, max_origin);

        XY::new(left + viewport_half, top + viewport_half)
    }

    fn tween_progress(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn tween_duration_ms(
        from: XY<f32>,
        to: XY<f32>,
        screen_spring: ScreenMovementSpring,
    ) -> f64 {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let responsiveness =
            (screen_spring.stiffness / (screen_spring.mass.max(0.1) * 140.0)).clamp(0.55, 2.2);
        let damping_bias = (screen_spring.damping / 14.0).clamp(0.75, 1.35);
        let duration = (110.0 + distance as f64 * 520.0) / (responsiveness * damping_bias) as f64;
        duration.clamp(85.0, 240.0)
    }

    fn evaluate_tween(tween: CameraShiftTween, time_ms: f64) -> XY<f32> {
        if time_ms <= tween.start_time_ms {
            return tween.from;
        }
        if time_ms >= tween.end_time_ms {
            return tween.to;
        }

        let t =
            ((time_ms - tween.start_time_ms) / (tween.end_time_ms - tween.start_time_ms)) as f32;
        let eased = Self::tween_progress(t);
        XY::new(
            tween.from.x + (tween.to.x - tween.from.x) * eased,
            tween.from.y + (tween.to.y - tween.from.y) * eased,
        )
    }

    fn raw_cursor_position_at(cursor_events: &CursorEvents, time_secs: f32) -> Option<XY<f32>> {
        let moves = &cursor_events.moves;
        if moves.is_empty() {
            return None;
        }

        let time_ms = time_secs as f64 * 1000.0;

        if time_ms <= moves[0].time_ms {
            return Some(XY::new(
                moves[0].x.clamp(0.0, 1.0) as f32,
                moves[0].y.clamp(0.0, 1.0) as f32,
            ));
        }

        if let Some(last) = moves.last()
            && time_ms >= last.time_ms
        {
            return Some(XY::new(
                last.x.clamp(0.0, 1.0) as f32,
                last.y.clamp(0.0, 1.0) as f32,
            ));
        }

        let idx = moves.partition_point(|m| m.time_ms <= time_ms);
        if idx == 0 {
            return Some(XY::new(
                moves[0].x.clamp(0.0, 1.0) as f32,
                moves[0].y.clamp(0.0, 1.0) as f32,
            ));
        }

        let prev = &moves[idx - 1];
        let next = &moves[idx.min(moves.len() - 1)];
        let dt = next.time_ms - prev.time_ms;

        const IDLE_GAP_MS: f64 = 66.67;
        if dt > IDLE_GAP_MS {
            return Some(XY::new(
                prev.x.clamp(0.0, 1.0) as f32,
                prev.y.clamp(0.0, 1.0) as f32,
            ));
        }

        let t = if dt > 1e-9 {
            ((time_ms - prev.time_ms) / dt) as f32
        } else {
            0.0
        };
        Some(XY::new(
            (prev.x as f32 + (next.x as f32 - prev.x as f32) * t).clamp(0.0, 1.0),
            (prev.y as f32 + (next.y as f32 - prev.y as f32) * t).clamp(0.0, 1.0),
        ))
    }

    pub fn ensure_precomputed_until(&mut self, time_secs: f32) {
        let duration_ms = self.duration_secs * 1000.0;
        let need_ms = (f64::from(time_secs) * 1000.0).clamp(0.0, duration_ms);

        if self.cursor_events.moves.is_empty() {
            if self.events.is_none() {
                self.events = Some(vec![]);
            }
            return;
        }

        if let Some(ref events) = self.events
            && let Some(last) = events.last()
            && last.time + f64::EPSILON >= need_ms
        {
            return;
        }

        if self.events.is_none() {
            let initial_pos = self.raw_focus_target_at(0.0);
            self.events = Some(vec![SmoothedFocusEvent {
                time: 0.0,
                position: initial_pos,
            }]);
            self.precompute_sim = Some(ZoomFocusPrecomputeSim {
                position: initial_pos,
                active_tween: None,
                last_integrated_ms: 0.0,
            });
        }

        loop {
            let next_ms = {
                let Some(ps) = self.precompute_sim.as_ref() else {
                    break;
                };
                if ps.last_integrated_ms + f64::EPSILON >= need_ms {
                    break;
                }
                let next_ms = (ps.last_integrated_ms + SAMPLE_INTERVAL_MS).min(duration_ms);
                if next_ms <= ps.last_integrated_ms + f64::EPSILON {
                    break;
                }
                next_ms
            };
            let time_secs = (next_ms / 1000.0) as f32;
            let raw_target = self.raw_focus_target_at(time_secs);
            let active_segment = self
                .active_segment_at(next_ms / 1000.0)
                .map(|segment| (segment.zoom_amount, segment.edge_snap_ratio));
            let Some(ps) = self.precompute_sim.as_mut() else {
                break;
            };
            let Some(events) = self.events.as_mut() else {
                break;
            };

            let current_position = ps
                .active_tween
                .map(|tween| Self::evaluate_tween(tween, ps.last_integrated_ms))
                .unwrap_or(ps.position);

            let desired_position = active_segment.map_or(raw_target, |(zoom_amount, edge_snap_ratio)| {
                Self::edge_shift_target(
                    current_position,
                    raw_target,
                    zoom_amount,
                    edge_snap_ratio,
                )
            });

            let delta_x = desired_position.x - current_position.x;
            let delta_y = desired_position.y - current_position.y;
            let shift_distance_sq = delta_x * delta_x + delta_y * delta_y;

            if shift_distance_sq > 0.000001 {
                let duration_ms =
                    Self::tween_duration_ms(current_position, desired_position, self.screen_spring);
                ps.active_tween = Some(CameraShiftTween {
                    from: current_position,
                    to: desired_position,
                    start_time_ms: ps.last_integrated_ms,
                    end_time_ms: ps.last_integrated_ms + duration_ms,
                });
            }

            let next_position = ps
                .active_tween
                .map(|tween| Self::evaluate_tween(tween, next_ms))
                .unwrap_or(current_position);

            if let Some(tween) = ps.active_tween
                && next_ms + f64::EPSILON >= tween.end_time_ms
            {
                ps.active_tween = None;
            }

            ps.last_integrated_ms = next_ms;
            ps.position = next_position;
            events.push(SmoothedFocusEvent {
                time: next_ms,
                position: XY::new(
                    next_position.x.clamp(0.0, 1.0),
                    next_position.y.clamp(0.0, 1.0),
                ),
            });
        }

        if let Some(ps) = self.precompute_sim.as_ref()
            && ps.last_integrated_ms + f64::EPSILON >= duration_ms
        {
            self.precompute_sim = None;
        }
    }

    pub fn precompute(&mut self) {
        self.ensure_precomputed_until(self.duration_secs as f32);
    }

    pub fn interpolate(&self, time_secs: f32) -> Coord<RawDisplayUVSpace> {
        let time_ms = (time_secs * 1000.0) as f64;

        if self.cursor_events.moves.is_empty() {
            return Coord::new(XY::new(0.5, 0.5));
        }

        if let Some(ref events) = self.events {
            self.interpolate_from_events(events, time_ms)
        } else {
            self.interpolate_direct(time_secs)
        }
    }

    fn interpolate_direct(&self, time_secs: f32) -> Coord<RawDisplayUVSpace> {
        let target = self.raw_focus_target_at(time_secs);
        Coord::new(XY::new(target.x as f64, target.y as f64))
    }

    fn interpolate_from_events(
        &self,
        events: &[SmoothedFocusEvent],
        time_ms: f64,
    ) -> Coord<RawDisplayUVSpace> {
        if events.is_empty() {
            return Coord::new(XY::new(0.5, 0.5));
        }

        if time_ms <= events[0].time {
            let pos = events[0].position;
            return Coord::new(XY::new(pos.x as f64, pos.y as f64));
        }

        if let Some(last) = events.last()
            && time_ms >= last.time
        {
            return Coord::new(XY::new(last.position.x as f64, last.position.y as f64));
        }

        let idx = events
            .binary_search_by(|e| {
                e.time
                    .partial_cmp(&time_ms)
                    .unwrap_or(std::cmp::Ordering::Less)
            })
            .unwrap_or_else(|i| i.saturating_sub(1));

        let curr = &events[idx];
        let next = events.get(idx + 1).unwrap_or(curr);

        if (next.time - curr.time).abs() < f64::EPSILON {
            return Coord::new(XY::new(curr.position.x as f64, curr.position.y as f64));
        }

        let t = ((time_ms - curr.time) / (next.time - curr.time)).clamp(0.0, 1.0) as f32;

        let lerped = XY::new(
            curr.position.x + (next.position.x - curr.position.x) * t,
            curr.position.y + (next.position.y - curr.position.y) * t,
        );

        Coord::new(XY::new(lerped.x as f64, lerped.y as f64))
    }
}
