use crate::common::Rect;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::screen_insets_scaled;
use crate::graphics::texture_set::TextureSet;

/// Touches that travel less than this many canvas pixels count as taps.
pub const TAP_SLOP: f64 = 8.0;
/// Canvas pixels between the buttons and the edge of the safe area.
const MARGIN: f32 = 4.0;
/// Side of a button, and of one d-pad cell, at size 1.0, in canvas pixels.
const CELL: f32 = 40.0;
/// Side of one icon in `builtin/touch`.
const ICON: f32 = 32.0;
/// Side of the round backing in `builtin/touch_plate`.
const PLATE: f32 = 48.0;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TouchControlType {
    None,
    Dialog,
    Controls,
}

/// The on-screen controls a player can move around.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TouchButton {
    DPad,
    Jump,
    Shoot,
    Inventory,
    Pause,
}

impl TouchButton {
    pub const ALL: [TouchButton; 5] =
        [TouchButton::DPad, TouchButton::Jump, TouchButton::Shoot, TouchButton::Inventory, TouchButton::Pause];

    /// Position of the button's icon in `builtin/touch`, in icon cells.
    fn icon(self) -> (u16, u16) {
        match self {
            TouchButton::DPad => (1, 1),
            TouchButton::Jump => (3, 1),
            TouchButton::Shoot => (3, 0),
            TouchButton::Inventory => (0, 3),
            TouchButton::Pause => (1, 3),
        }
    }
}

/// Look and placement of the on-screen controls, saved with the settings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TouchLayout {
    /// 0.0 (invisible) to 1.0.
    pub opacity: f32,
    /// Multiplier on the default button size.
    pub size: f32,
    /// Button centres as fractions of the safe area; `None` keeps the default spot.
    pub dpad: Option<(f32, f32)>,
    pub jump: Option<(f32, f32)>,
    pub shoot: Option<(f32, f32)>,
    pub inventory: Option<(f32, f32)>,
    pub pause: Option<(f32, f32)>,
}

impl Default for TouchLayout {
    fn default() -> Self {
        TouchLayout { opacity: 0.75, size: 1.0, dpad: None, jump: None, shoot: None, inventory: None, pause: None }
    }
}

impl TouchLayout {
    pub const OPACITIES: [f32; 4] = [0.25, 0.5, 0.75, 1.0];
    pub const SIZES: [f32; 4] = [0.75, 1.0, 1.25, 1.5];

    pub fn position(&self, button: TouchButton) -> Option<(f32, f32)> {
        match button {
            TouchButton::DPad => self.dpad,
            TouchButton::Jump => self.jump,
            TouchButton::Shoot => self.shoot,
            TouchButton::Inventory => self.inventory,
            TouchButton::Pause => self.pause,
        }
    }

    pub fn set_position(&mut self, button: TouchButton, position: Option<(f32, f32)>) {
        match button {
            TouchButton::DPad => self.dpad = position,
            TouchButton::Jump => self.jump = position,
            TouchButton::Shoot => self.shoot = position,
            TouchButton::Inventory => self.inventory = position,
            TouchButton::Pause => self.pause = position,
        }
    }

    pub fn reset_positions(&mut self) {
        for button in TouchButton::ALL {
            self.set_position(button, None);
        }
    }

    /// Index of the closest entry of `options` to `value`.
    pub fn closest_index(options: &[f32], value: f32) -> usize {
        options
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| (**a - value).abs().total_cmp(&(**b - value).abs()))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }
}

/// Where the controls sit on the canvas for the current screen and layout.
#[derive(Copy, Clone)]
pub struct TouchGeometry {
    /// Safe area edges in canvas pixels: left, top, right, bottom.
    area: (f32, f32, f32, f32),
    cell: f32,
}

impl TouchGeometry {
    pub fn new(ctx: &mut Context, canvas_size: (f32, f32), scale: f32, layout: &TouchLayout) -> TouchGeometry {
        let (left, top, right, bottom) = screen_insets_scaled(ctx, scale);
        TouchGeometry {
            area: (left, top, canvas_size.0 - right, canvas_size.1 - bottom),
            cell: CELL * layout.size.max(0.25),
        }
    }

    pub fn area(&self) -> (f32, f32, f32, f32) {
        self.area
    }

    fn extent(&self, button: TouchButton) -> f32 {
        match button {
            TouchButton::DPad => self.cell * 3.0,
            _ => self.cell,
        }
    }

    fn default_center(&self, button: TouchButton) -> (f32, f32) {
        let (left, top, right, bottom) = self.area;
        let (c, m) = (self.cell, MARGIN);

        match button {
            TouchButton::DPad => (left + m + c * 1.5, bottom - m - c * 1.5),
            TouchButton::Jump => (right - m - c * 0.5, bottom - m - c * 0.5),
            TouchButton::Shoot => (right - m - c * 0.5, bottom - m * 2.0 - c * 1.5),
            TouchButton::Inventory => (right - m - c * 0.5, top + m + c * 0.5),
            // Next to Inventory rather than top-left, where it covered the weapon HUD.
            TouchButton::Pause => (right - m * 2.0 - c * 1.5, top + m + c * 0.5),
        }
    }

    fn clamp(&self, button: TouchButton, x: f32, y: f32) -> (f32, f32) {
        let half = self.extent(button) / 2.0;
        let (left, top, right, bottom) = self.area;

        (x.clamp(left + half, (right - half).max(left + half)), y.clamp(top + half, (bottom - half).max(top + half)))
    }

    pub fn center(&self, layout: &TouchLayout, button: TouchButton) -> (f32, f32) {
        let (left, top, right, bottom) = self.area;
        let (x, y) = match layout.position(button) {
            Some((fx, fy)) => (left + fx * (right - left), top + fy * (bottom - top)),
            None => self.default_center(button),
        };

        self.clamp(button, x, y)
    }

    /// The layout fractions that put `button`'s centre at (x, y), kept on screen.
    pub fn to_fraction(&self, button: TouchButton, x: f32, y: f32) -> (f32, f32) {
        let (x, y) = self.clamp(button, x, y);
        let (left, top, right, bottom) = self.area;

        ((x - left) / (right - left).max(1.0), (y - top) / (bottom - top).max(1.0))
    }

    pub fn rect(&self, layout: &TouchLayout, button: TouchButton) -> Rect<isize> {
        let (x, y) = self.center(layout, button);
        let side = self.extent(button);

        Rect::new_size((x - side / 2.0) as isize, (y - side / 2.0) as isize, side as isize, side as isize)
    }

    /// D-pad directions (left, up, right, down) held by a touch. A touch that started on the pad keeps
    /// steering when the thumb drifts past its edge; other touches must be on the pad itself.
    /// Straight directions get wider slices than diagonals, so walking rarely presses down by accident.
    pub fn dpad_directions(&self, layout: &TouchLayout, point: &TouchPoint) -> Option<(bool, bool, bool, bool)> {
        let (cx, cy) = self.center(layout, TouchButton::DPad);
        let started_on_pad = point.first_position_in(self.rect(layout, TouchButton::DPad));
        let reach = if started_on_pad { self.cell * 2.5 } else { self.cell * 1.5 };

        let (dx, dy) = (point.position.0 as f32 - cx, point.position.1 as f32 - cy);
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > reach {
            return None;
        }
        if distance < self.cell * 0.25 {
            return Some((false, false, false, false));
        }

        // Degrees clockwise from "right", as y grows downwards.
        let angle = dy.atan2(dx).to_degrees();
        Some((angle.abs() >= 120.0, (-150.0..=-30.0).contains(&angle), angle.abs() <= 60.0, (30.0..=150.0).contains(&angle)))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TouchPoint {
    pub id: u64,
    pub touch_id: u64,
    pub position: (f64, f64),
    pub first_position: (f64, f64),
    pub last_position: (f64, f64),
}

impl TouchPoint {
    pub fn is_in(&self, bounds: Rect) -> bool {
        Self::contains(bounds, self.position)
    }

    pub fn first_position_in(&self, bounds: Rect) -> bool {
        Self::contains(bounds, self.first_position)
    }

    /// How far the touch has travelled from where it went down, in canvas pixels.
    pub fn travel(&self) -> f64 {
        let (dx, dy) = (self.position.0 - self.first_position.0, self.position.1 - self.first_position.1);
        (dx * dx + dy * dy).sqrt()
    }

    fn contains(bounds: Rect, (x, y): (f64, f64)) -> bool {
        (x as isize) > bounds.left && (x as isize) < bounds.right && (y as isize) > bounds.top && (y as isize) < bounds.bottom
    }
}

/// Platform-agnostic touch event delivered by backends.
#[derive(Copy, Clone, Debug)]
pub enum TouchEvent {
    Down { id: u64, x: f64, y: f64 },
    Move { id: u64, x: f64, y: f64 },
    Up { id: u64 },
}

pub struct TouchControls {
    pub control_type: TouchControlType,
    pub points: Vec<TouchPoint>,
    pub interact_icon: bool,
    pub touch_id_counter: u64,
    /// Touches that just went down; gameplay reacts on press.
    pub clicks: Vec<TouchPoint>,
    /// Touches that just lifted without dragging; menus react on release so a swipe can scroll them.
    pub taps: Vec<(f64, f64)>,
}

impl TouchControls {
    pub fn new() -> TouchControls {
        TouchControls {
            control_type: TouchControlType::None,
            points: Vec::with_capacity(8),
            interact_icon: false,
            touch_id_counter: 0,
            clicks: Vec::with_capacity(8),
            taps: Vec::with_capacity(8),
        }
    }

    pub fn find_point_in(&self, bounds: Rect) -> Option<TouchPoint> {
        self.points.iter().find(|p| p.is_in(bounds)).map(|p| *p)
    }

    pub fn point_in(&self, bounds: Rect) -> Option<u64> {
        self.find_point_in(bounds).map(|p| p.touch_id)
    }

    /// Apply a platform-agnostic touch event, updating points/clicks bookkeeping.
    pub fn apply_event(&mut self, ev: TouchEvent) {
        match ev {
            TouchEvent::Down { id, x, y } | TouchEvent::Move { id, x, y } => {
                if let Some(point) = self.points.iter_mut().find(|p| p.id == id) {
                    point.last_position = point.position;
                    point.position = (x, y);
                } else {
                    self.touch_id_counter = self.touch_id_counter.wrapping_add(1);
                    let point = TouchPoint {
                        id,
                        touch_id: self.touch_id_counter,
                        position: (x, y),
                        first_position: (x, y),
                        last_position: (0.0, 0.0),
                    };
                    self.points.push(point);
                    if matches!(ev, TouchEvent::Down { .. }) {
                        self.clicks.push(point);
                    }
                }
            }
            TouchEvent::Up { id } => {
                if let Some(point) = self.points.iter().find(|p| p.id == id) {
                    if point.travel() < TAP_SLOP {
                        self.taps.push(point.position);
                    }
                }
                self.points.retain(|p| p.id != id);
                self.clicks.retain(|p| p.id != id);
            }
        }
    }

    pub fn consume_click_in(&mut self, bounds: Rect) -> bool {
        self.clicks.retain(|p| p.touch_id != 0);

        if let Some(point) = self.clicks.iter_mut().find(|p| p.is_in(bounds)) {
            point.touch_id = 0;

            return true;
        }

        false
    }

    pub fn consume_tap_in(&mut self, bounds: Rect) -> bool {
        if let Some(index) = self.taps.iter().position(|&tap| TouchPoint::contains(bounds, tap)) {
            self.taps.remove(index);
            return true;
        }

        false
    }

    /// Drops taps nothing consumed during the tick, so they can't fire later somewhere else.
    pub fn end_tick(&mut self) {
        self.taps.clear();
    }

    pub fn draw(
        &self,
        canvas_size: (f32, f32),
        scale: f32,
        layout: &TouchLayout,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        let geometry = TouchGeometry::new(ctx, canvas_size, scale, layout);

        match self.control_type {
            TouchControlType::None => Ok(()),
            // Fast-forward sits where Inventory does, and is pressed in the same spot.
            TouchControlType::Dialog => {
                self.draw_button(&geometry, layout, TouchButton::Inventory, (2, 3), false, constants, texture_set, ctx)
            }
            TouchControlType::Controls => self.draw_buttons(&geometry, layout, None, constants, texture_set, ctx),
        }
    }

    /// Draws every movable control. `highlight` marks one as held, for the layout editor.
    pub fn draw_buttons(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        highlight: Option<TouchButton>,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        self.draw_dpad(geometry, layout, highlight == Some(TouchButton::DPad), constants, texture_set, ctx)?;

        for button in [TouchButton::Jump, TouchButton::Shoot, TouchButton::Inventory, TouchButton::Pause] {
            let held = highlight == Some(button) || self.find_point_in(geometry.rect(layout, button)).is_some();
            self.draw_button(geometry, layout, button, button.icon(), held, constants, texture_set, ctx)?;
        }

        Ok(())
    }

    fn colors(layout: &TouchLayout, held: bool) -> ((u8, u8, u8, u8), (u8, u8, u8, u8)) {
        let alpha = (layout.opacity.clamp(0.0, 1.0) * 255.0) as u8;
        if held {
            ((255, 220, 120, alpha), (255, 240, 160, alpha))
        } else {
            ((255, 255, 255, alpha), (255, 255, 255, alpha))
        }
    }

    fn draw_button(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        button: TouchButton,
        icon: (u16, u16),
        held: bool,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        let (plate_color, icon_color) = Self::colors(layout, held);
        let rect = geometry.rect(layout, button);
        let side = rect.width() as f32;
        let icon_side = side * ICON / CELL;

        let plates = texture_set.get_or_load_batch(ctx, constants, "builtin/touch_plate")?;
        plates.add_rect_scaled_tinted(
            rect.left as f32,
            rect.top as f32,
            plate_color,
            side / PLATE,
            side / PLATE,
            &Rect::new_size(0, 0, PLATE as u16, PLATE as u16),
        );
        plates.draw(ctx)?;

        let icons = texture_set.get_or_load_batch(ctx, constants, "builtin/touch")?;
        icons.add_rect_scaled_tinted(
            rect.left as f32 + (side - icon_side) / 2.0,
            rect.top as f32 + (side - icon_side) / 2.0,
            icon_color,
            icon_side / ICON,
            icon_side / ICON,
            &Rect::new_size(icon.0 * 32, icon.1 * 32, 32, 32),
        );
        icons.draw(ctx)
    }

    fn draw_dpad(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        highlight: bool,
        constants: &EngineConstants,
        texture_set: &mut TextureSet,
        ctx: &mut Context,
    ) -> GameResult {
        let (plate_color, _) = Self::colors(layout, highlight);
        let rect = geometry.rect(layout, TouchButton::DPad);
        let side = rect.width() as f32;
        let cell = side / 3.0;
        let icon_side = cell * ICON / CELL;

        let (mut left, mut up, mut right, mut down) = (false, false, false, false);
        for point in &self.points {
            if let Some((l, u, r, d)) = geometry.dpad_directions(layout, point) {
                left |= l;
                up |= u;
                right |= r;
                down |= d;
            }
        }

        let held_cell = |x: u16, y: u16| {
            let required = (x != 0 || left) && (x != 2 || right) && (y != 0 || up) && (y != 2 || down);
            let exact = (x != 1 || !(left || right)) && (y != 1 || !(up || down));
            required && exact && (left || up || right || down)
        };

        let plates = texture_set.get_or_load_batch(ctx, constants, "builtin/touch_plate")?;
        plates.add_rect_scaled_tinted(
            rect.left as f32,
            rect.top as f32,
            plate_color,
            side / PLATE,
            side / PLATE,
            &Rect::new_size(0, 0, PLATE as u16, PLATE as u16),
        );
        // A small lit disc under the held direction.
        let (held_plate, _) = Self::colors(layout, true);
        for x in 0..3u16 {
            for y in 0..3u16 {
                if held_cell(x, y) {
                    plates.add_rect_scaled_tinted(
                        rect.left as f32 + cell * x as f32,
                        rect.top as f32 + cell * y as f32,
                        held_plate,
                        cell / PLATE,
                        cell / PLATE,
                        &Rect::new_size(0, 0, PLATE as u16, PLATE as u16),
                    );
                }
            }
        }
        plates.draw(ctx)?;

        let icons = texture_set.get_or_load_batch(ctx, constants, "builtin/touch")?;
        for x in 0..3u16 {
            for y in 0..3u16 {
                if x == 1 && y == 1 {
                    continue;
                }

                let icon_x = if self.interact_icon && x == 1 && y == 2 { 3 } else { x };
                let (_, icon_color) = Self::colors(layout, held_cell(x, y));

                icons.add_rect_scaled_tinted(
                    rect.left as f32 + cell * x as f32 + (cell - icon_side) / 2.0,
                    rect.top as f32 + cell * y as f32 + (cell - icon_side) / 2.0,
                    icon_color,
                    icon_side / ICON,
                    icon_side / ICON,
                    &Rect::new_size(icon_x * 32, y * 32, 32, 32),
                );
            }
        }
        icons.draw(ctx)
    }
}
