use crate::common::Rect;
use crate::engine_constants::EngineConstants;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::screen_insets_scaled;
use crate::graphics::bmfont::BMFont;
use crate::graphics::font::Font;
use crate::graphics::texture_set::TextureSet;
use crate::i18n::Locale;

/// Touches that travel less than this many canvas pixels count as taps.
pub const TAP_SLOP: f64 = 8.0;
/// Canvas pixels between the buttons and the edge of the safe area.
const MARGIN: f32 = 4.0;
/// Side of a button, and of one d-pad cell, at size 1.0, in canvas pixels.
const CELL: f32 = 40.0;
/// Side of one icon in `builtin/touch_icons`.
const ICON: f32 = 32.0;
/// Side of one backing in `builtin/touch_plate`.
const PLATE: f32 = 48.0;
/// Below the weapon and health HUD, where the weapon buttons start by default.
const HUD_HEIGHT: f32 = 52.0;

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
    PrevWeapon,
    NextWeapon,
    Map,
}

impl TouchButton {
    pub const ALL: [TouchButton; 8] = [
        TouchButton::DPad,
        TouchButton::Jump,
        TouchButton::Shoot,
        TouchButton::Inventory,
        TouchButton::Pause,
        TouchButton::PrevWeapon,
        TouchButton::NextWeapon,
        TouchButton::Map,
    ];

    /// The button's icon: its cell in the 4x4 icon layout, or one of the weapon icons in row 8.
    fn icon(self) -> Icon {
        match self {
            TouchButton::DPad => Icon::Cell(1, 1),
            TouchButton::Jump => Icon::Cell(3, 1),
            TouchButton::Shoot => Icon::Cell(3, 0),
            TouchButton::Inventory => Icon::Cell(0, 3),
            TouchButton::Pause => Icon::Cell(1, 3),
            TouchButton::PrevWeapon => Icon::Weapon(false),
            TouchButton::NextWeapon => Icon::Weapon(true),
            TouchButton::Map => Icon::Map,
        }
    }

    /// Short label for the Text icon style.
    fn text_key(self) -> &'static str {
        match self {
            TouchButton::DPad => "menus.controls_menu.touch_menu.text.dpad",
            TouchButton::Jump => "menus.controls_menu.touch_menu.text.jump",
            TouchButton::Shoot => "menus.controls_menu.touch_menu.text.shoot",
            TouchButton::Inventory => "menus.controls_menu.touch_menu.text.inventory",
            TouchButton::Pause => "menus.controls_menu.touch_menu.text.pause",
            TouchButton::PrevWeapon => "menus.controls_menu.touch_menu.text.prev_weapon",
            TouchButton::NextWeapon => "menus.controls_menu.touch_menu.text.next_weapon",
            TouchButton::Map => "menus.controls_menu.touch_menu.text.map",
        }
    }
}

#[derive(Copy, Clone)]
enum Icon {
    Cell(u16, u16),
    Weapon(bool),
    Map,
}

/// Declares a settings enum with its menu order and the locale key of each value.
macro_rules! touch_option {
    ($(#[$doc:meta])* $name:ident { $($variant:ident => $key:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// Last part of the value's locale key.
            pub fn key(self) -> &'static str {
                match self {
                    $($name::$variant => $key),+
                }
            }

            pub fn index(self) -> usize {
                Self::ALL.iter().position(|v| *v == self).unwrap_or(0)
            }
        }
    };
}

touch_option!(
    /// Which icons the buttons show.
    TouchIconStyle {
        Classic => "classic",
        Bold => "bold",
        Text => "text",
    }
);

touch_option!(
    /// Colour of the icons and rims; the backing stays dark.
    TouchColor {
        Ice => "ice",
        White => "white",
        Red => "red",
        Gold => "gold",
        Green => "green",
        Purple => "purple",
        Pink => "pink",
    }
);

touch_option!(
    /// Backing behind each button.
    TouchShape {
        Round => "round",
        Square => "square",
        None => "none",
    }
);

touch_option!(
    /// How the d-pad looks; it steers the same either way.
    TouchDpadStyle {
        Arrows => "arrows",
        Stick => "stick",
    }
);

impl TouchColor {
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            TouchColor::Ice => (170, 195, 255),
            TouchColor::White => (255, 255, 255),
            TouchColor::Red => (255, 96, 86),
            TouchColor::Gold => (255, 205, 70),
            TouchColor::Green => (120, 235, 120),
            TouchColor::Purple => (195, 140, 255),
            TouchColor::Pink => (255, 145, 205),
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
    pub icon_style: TouchIconStyle,
    pub color: TouchColor,
    pub shape: TouchShape,
    pub dpad_style: TouchDpadStyle,
    /// Whether the previous/next weapon buttons are shown.
    pub weapon_buttons: bool,
    /// Whether the Map button is shown.
    pub map_button: bool,
    /// The d-pad appears wherever the left thumb lands on the left half of the screen.
    pub floating_stick: bool,
    /// Buttons' touch areas reach a little past their drawings.
    pub forgiving_buttons: bool,
    /// A light tick when an on-screen button is pressed (phones only).
    pub button_taps: bool,
    /// Haptic taps when the player is hurt and during quakes (phones only).
    pub vibration: bool,
    /// Button centres as fractions of the safe area; `None` keeps the default spot.
    pub dpad: Option<(f32, f32)>,
    pub jump: Option<(f32, f32)>,
    pub shoot: Option<(f32, f32)>,
    pub inventory: Option<(f32, f32)>,
    pub pause: Option<(f32, f32)>,
    pub prev_weapon: Option<(f32, f32)>,
    pub next_weapon: Option<(f32, f32)>,
    pub map: Option<(f32, f32)>,
}

impl Default for TouchLayout {
    fn default() -> Self {
        TouchLayout {
            opacity: 0.75,
            size: 1.0,
            icon_style: TouchIconStyle::Classic,
            color: TouchColor::Ice,
            shape: TouchShape::Round,
            dpad_style: TouchDpadStyle::Arrows,
            weapon_buttons: true,
            map_button: true,
            floating_stick: false,
            forgiving_buttons: true,
            button_taps: true,
            vibration: true,
            dpad: None,
            jump: None,
            shoot: None,
            inventory: None,
            pause: None,
            prev_weapon: None,
            next_weapon: None,
            map: None,
        }
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
            TouchButton::PrevWeapon => self.prev_weapon,
            TouchButton::NextWeapon => self.next_weapon,
            TouchButton::Map => self.map,
        }
    }

    pub fn set_position(&mut self, button: TouchButton, position: Option<(f32, f32)>) {
        match button {
            TouchButton::DPad => self.dpad = position,
            TouchButton::Jump => self.jump = position,
            TouchButton::Shoot => self.shoot = position,
            TouchButton::Inventory => self.inventory = position,
            TouchButton::Pause => self.pause = position,
            TouchButton::PrevWeapon => self.prev_weapon = position,
            TouchButton::NextWeapon => self.next_weapon = position,
            TouchButton::Map => self.map = position,
        }
    }

    pub fn reset_positions(&mut self) {
        for button in TouchButton::ALL {
            self.set_position(button, None);
        }
    }

    /// The buttons on screen: all of them, minus the weapon and map buttons when they're turned off.
    pub fn visible_buttons(&self) -> Vec<TouchButton> {
        TouchButton::ALL
            .iter()
            .copied()
            .filter(|b| match b {
                TouchButton::PrevWeapon | TouchButton::NextWeapon => self.weapon_buttons,
                TouchButton::Map => self.map_button,
                _ => true,
            })
            .collect()
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
            TouchButton::PrevWeapon | TouchButton::NextWeapon => self.cell * 0.8,
            _ => self.cell,
        }
    }

    fn default_center(&self, button: TouchButton) -> (f32, f32) {
        let (left, top, right, bottom) = self.area;
        let (c, m) = (self.cell, MARGIN);
        let w = self.extent(TouchButton::PrevWeapon);

        match button {
            TouchButton::DPad => (left + m + c * 1.5, bottom - m - c * 1.5),
            TouchButton::Jump => (right - m - c * 0.5, bottom - m - c * 0.5),
            TouchButton::Shoot => (right - m - c * 0.5, bottom - m * 2.0 - c * 1.5),
            TouchButton::Inventory => (right - m - c * 0.5, top + m + c * 0.5),
            // Next to Inventory rather than top-left, where it covered the weapon HUD.
            TouchButton::Pause => (right - m * 2.0 - c * 1.5, top + m + c * 0.5),
            // Just under the weapon HUD they switch.
            TouchButton::PrevWeapon => (left + m + w * 0.5, top + HUD_HEIGHT + w * 0.5),
            TouchButton::NextWeapon => (left + m * 2.0 + w * 1.5, top + HUD_HEIGHT + w * 0.5),
            // Left of Pause, with the other menu buttons.
            TouchButton::Map => (right - m * 3.0 - c * 2.5, top + m + c * 0.5),
        }
    }

    pub fn clamp(&self, button: TouchButton, x: f32, y: f32) -> (f32, f32) {
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

    /// The centre a touch steers the d-pad around and how far from it the touch still counts, or `None`
    /// when the touch isn't steering it. A touch that started on the pad keeps steering when the thumb
    /// drifts past its edge; other touches must be on the pad itself. With the floating stick, any touch
    /// that starts on the left half of the screen (and not on another button) steers around where it went down.
    pub fn dpad_anchor(&self, layout: &TouchLayout, point: &TouchPoint) -> Option<(f32, f32, f32)> {
        if layout.floating_stick {
            let (left, _, right, _) = self.area;
            let (x, y) = (point.first_position.0 as f32, point.first_position.1 as f32);
            let on_other_button = layout
                .visible_buttons()
                .into_iter()
                .any(|button| button != TouchButton::DPad && point.first_position_in(self.rect(layout, button)));

            return (x < (left + right) / 2.0 && !on_other_button).then_some((x, y, self.cell * 3.0));
        }

        let (cx, cy) = self.center(layout, TouchButton::DPad);
        let reach = if point.first_position_in(self.rect(layout, TouchButton::DPad)) {
            self.cell * 2.5
        } else {
            self.cell * 1.5
        };

        Some((cx, cy, reach))
    }

    /// D-pad directions (left, up, right, down) held by a touch (see `dpad_anchor`).
    /// Straight directions get wider slices than diagonals, so walking rarely presses down by accident.
    pub fn dpad_directions(&self, layout: &TouchLayout, point: &TouchPoint) -> Option<(bool, bool, bool, bool)> {
        let (cx, cy, reach) = self.dpad_anchor(layout, point)?;

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

    /// How far a button's touch area reaches past its drawing.
    fn hit_margin(&self, layout: &TouchLayout) -> f32 {
        if layout.forgiving_buttons {
            self.cell * 0.25
        } else {
            0.0
        }
    }

    /// A button's touch area: its drawing, plus a margin with forgiving buttons on.
    pub fn hit_rect(&self, layout: &TouchLayout, button: TouchButton) -> Rect<isize> {
        let rect = self.rect(layout, button);
        let margin = self.hit_margin(layout) as isize;
        Rect::new(rect.left - margin, rect.top - margin, rect.right + margin, rect.bottom + margin)
    }

    /// The button (other than the d-pad) under a touch at (x, y). Where enlarged touch areas overlap,
    /// the closest button wins, so one press never hits two. The Map button counts only once owned.
    pub fn button_at(&self, layout: &TouchLayout, x: f32, y: f32, has_map: bool) -> Option<TouchButton> {
        layout
            .visible_buttons()
            .into_iter()
            .filter(|&button| button != TouchButton::DPad && (button != TouchButton::Map || has_map))
            .filter(|&button| TouchPoint::contains(self.hit_rect(layout, button), (x as f64, y as f64)))
            .map(|button| {
                let (cx, cy) = self.center(layout, button);
                (button, (x - cx) * (x - cx) + (y - cy) * (y - cy))
            })
            .min_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(button, _)| button)
    }
}

/// What drawing the controls needs from the game state.
pub struct TouchArt<'a> {
    pub constants: &'a EngineConstants,
    pub texture_set: &'a mut TextureSet,
    pub font: &'a BMFont,
    pub loc: &'a Locale,
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
    /// Whether the player owns the Map System; the Map button only shows in game once they do.
    pub has_map: bool,
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
            has_map: false,
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

    /// Like `consume_click_in`, for a press that `hit` accepts.
    pub fn consume_click_where(&mut self, hit: impl Fn(&TouchPoint) -> bool) -> bool {
        self.clicks.retain(|p| p.touch_id != 0);

        if let Some(point) = self.clicks.iter_mut().find(|p| hit(p)) {
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
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        let geometry = TouchGeometry::new(ctx, canvas_size, scale, layout);

        match self.control_type {
            TouchControlType::None => Ok(()),
            // Fast-forward sits where Inventory does, and is pressed in the same spot.
            TouchControlType::Dialog => self.draw_button(
                &geometry,
                layout,
                TouchButton::Inventory,
                Icon::Cell(2, 3),
                "menus.controls_menu.touch_menu.text.skip",
                false,
                art,
                ctx,
            ),
            TouchControlType::Controls => self.draw_buttons(&geometry, layout, None, true, art, ctx),
        }
    }

    /// Draws every visible control. `highlight` marks one as held, for the layout editor. `in_game` leaves
    /// out the Map button until the Map System is owned (menus show it so it can be placed early).
    pub fn draw_buttons(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        highlight: Option<TouchButton>,
        in_game: bool,
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        for button in layout.visible_buttons() {
            if in_game && button == TouchButton::Map && !self.has_map {
                continue;
            }

            if button == TouchButton::DPad {
                self.draw_dpad(geometry, layout, highlight == Some(button), art, ctx)?;
            } else {
                let held = highlight == Some(button)
                    || self.points.iter().any(|p| {
                        geometry.dpad_anchor(layout, p).filter(|_| layout.floating_stick).is_none()
                            && geometry.button_at(layout, p.position.0 as f32, p.position.1 as f32, !in_game || self.has_map)
                                == Some(button)
                    });
                self.draw_button(geometry, layout, button, button.icon(), button.text_key(), held, art, ctx)?;
            }
        }

        Ok(())
    }

    fn alpha(layout: &TouchLayout, fraction: f32) -> u8 {
        (layout.opacity.clamp(0.0, 1.0) * fraction * 255.0) as u8
    }

    /// The backing's cell in `builtin/touch_plate`: (x, y) of the rimmed version; the solid one is 48px right.
    fn plate_origin(shape: TouchShape) -> (u16, u16) {
        match shape {
            TouchShape::Square => (0, 48),
            _ => (0, 0),
        }
    }

    /// Draws the backing (when asked and the shape isn't None) and, when held, a coloured glow over it.
    fn draw_plate(
        layout: &TouchLayout,
        x: f32,
        y: f32,
        side: f32,
        backing: bool,
        held: bool,
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        let (r, g, b) = layout.color.rgb();
        let (px, py) = Self::plate_origin(layout.shape);
        let batch = art.texture_set.get_or_load_batch(ctx, art.constants, "builtin/touch_plate")?;

        if backing && layout.shape != TouchShape::None {
            batch.add_rect_scaled_tinted(
                x,
                y,
                (r, g, b, Self::alpha(layout, 1.0)),
                side / PLATE,
                side / PLATE,
                &Rect::new_size(px, py, PLATE as u16, PLATE as u16),
            );
        }
        if held {
            batch.add_rect_scaled_tinted(
                x,
                y,
                (r, g, b, Self::alpha(layout, 0.45)),
                side / PLATE,
                side / PLATE,
                &Rect::new_size(px + PLATE as u16, py, PLATE as u16, PLATE as u16),
            );
        }

        batch.draw(ctx)
    }

    /// Draws an icon centred in a square of `side`, white while held and in the chosen colour otherwise.
    fn draw_icon(
        layout: &TouchLayout,
        icon: Icon,
        x: f32,
        y: f32,
        side: f32,
        held: bool,
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        let (r, g, b) = if held { (255, 255, 255) } else { layout.color.rgb() };
        let bold = layout.icon_style != TouchIconStyle::Classic;
        let source = match icon {
            Icon::Cell(cx, cy) => Rect::new_size(cx * 32, cy * 32 + if bold { 128 } else { 0 }, 32, 32),
            Icon::Weapon(next) => Rect::new_size(if bold { 64 } else { 0 } + if next { 32 } else { 0 }, 256, 32, 32),
            Icon::Map => Rect::new_size(if bold { 32 } else { 0 }, 288, 32, 32),
        };
        let icon_side = side * ICON / CELL;

        let batch = art.texture_set.get_or_load_batch(ctx, art.constants, "builtin/touch_icons")?;
        batch.add_rect_scaled_tinted(
            x + (side - icon_side) / 2.0,
            y + (side - icon_side) / 2.0,
            (r, g, b, Self::alpha(layout, 1.0)),
            icon_side / ICON,
            icon_side / ICON,
            &source,
        );
        batch.draw(ctx)
    }

    fn draw_button(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        button: TouchButton,
        icon: Icon,
        text_key: &str,
        held: bool,
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        let rect = geometry.rect(layout, button);
        let (x, y, side) = (rect.left as f32, rect.top as f32, rect.width() as f32);

        Self::draw_plate(layout, x, y, side, true, held, art, ctx)?;

        if layout.icon_style == TouchIconStyle::Text {
            let (r, g, b) = if held { (255, 255, 255) } else { layout.color.rgb() };
            let text = art.loc.t(text_key);
            let width = art.font.builder().compute_width(text);
            let line_height = art.font.line_height();

            art.font
                .builder()
                .position(x + (side - width) / 2.0, y + (side - line_height) / 2.0)
                .shadow(true)
                .color((r, g, b, Self::alpha(layout, 1.0)))
                .draw(text, ctx, art.constants, art.texture_set)
        } else {
            Self::draw_icon(layout, icon, x, y, side, held, art, ctx)
        }
    }

    fn draw_dpad(
        &self,
        geometry: &TouchGeometry,
        layout: &TouchLayout,
        highlight: bool,
        art: &mut TouchArt,
        ctx: &mut Context,
    ) -> GameResult {
        // The touch steering the pad, if any, with the centre it steers around. A fixed pad follows only
        // touches that started on it; others still press directions but don't move the knob.
        let steering = self.points.iter().find_map(|point| {
            let (ax, ay, _) = geometry.dpad_anchor(layout, point)?;
            let dirs = geometry.dpad_directions(layout, point)?;
            (layout.floating_stick || point.first_position_in(geometry.rect(layout, TouchButton::DPad)))
                .then_some((point, (ax, ay), dirs))
        });

        // The floating stick sits under the thumb while steering, and shows faded at its usual spot otherwise.
        let (cx, cy) = match steering {
            Some((_, (ax, ay), _)) if layout.floating_stick => geometry.clamp(TouchButton::DPad, ax, ay),
            _ => geometry.center(layout, TouchButton::DPad),
        };
        let faded;
        let layout = if layout.floating_stick && steering.is_none() {
            faded = TouchLayout { opacity: layout.opacity * 0.5, ..layout.clone() };
            &faded
        } else {
            layout
        };

        let side = geometry.rect(layout, TouchButton::DPad).width() as f32;
        let (x, y) = (cx - side / 2.0, cy - side / 2.0);
        let cell = side / 3.0;

        Self::draw_plate(layout, x, y, side, true, highlight, art, ctx)?;

        if layout.dpad_style == TouchDpadStyle::Stick {
            // A knob that follows the thumb, kept inside the pad.
            let knob = side * 0.42;
            let (mut dx, mut dy) = steering
                .map(|(p, (ax, ay), _)| (p.position.0 as f32 - ax, p.position.1 as f32 - ay))
                .unwrap_or((0.0, 0.0));
            let reach = (side - knob) / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > reach {
                dx *= reach / distance;
                dy *= reach / distance;
            }

            let (kx, ky) = (cx + dx - knob / 2.0, cy + dy - knob / 2.0);
            let shape = TouchLayout { shape: TouchShape::Round, ..layout.clone() };
            Self::draw_plate(&shape, kx, ky, knob, true, true, art, ctx)?;
            if self.interact_icon {
                Self::draw_icon(layout, Icon::Cell(3, 2), kx, ky, knob, false, art, ctx)?;
            }
            return Ok(());
        }

        let (left, up, right, down) = steering.map(|(_, _, dirs)| dirs).unwrap_or((false, false, false, false));
        let held_cell = |x: u16, y: u16| {
            let required = (x != 0 || left) && (x != 2 || right) && (y != 0 || up) && (y != 2 || down);
            let exact = (x != 1 || !(left || right)) && (y != 1 || !(up || down));
            required && exact && (left || up || right || down)
        };

        // Text labels can't point, so that style uses the bold arrows on the pad.
        let arrows = TouchLayout {
            icon_style: if layout.icon_style == TouchIconStyle::Classic {
                TouchIconStyle::Classic
            } else {
                TouchIconStyle::Bold
            },
            ..layout.clone()
        };

        for cx in 0..3u16 {
            for cy in 0..3u16 {
                if cx == 1 && cy == 1 {
                    continue;
                }

                let (mut ix, mut iy) = (x + cell * cx as f32, y + cell * cy as f32);
                // Corners of a square pad fall outside a round backing; pull the diagonals in.
                if cx != 1 && cy != 1 && layout.shape != TouchShape::Square {
                    ix += (1.0 - cx as f32) * cell * 0.2;
                    iy += (1.0 - cy as f32) * cell * 0.2;
                }
                let held = held_cell(cx, cy);
                if held {
                    // Only the glow: the pad already has its backing.
                    Self::draw_plate(&arrows, ix, iy, cell, false, true, art, ctx)?;
                }

                let icon = if self.interact_icon && cx == 1 && cy == 2 { Icon::Cell(3, 2) } else { Icon::Cell(cx, cy) };
                Self::draw_icon(&arrows, icon, ix, iy, cell, held, art, ctx)?;
            }
        }

        Ok(())
    }
}
