use crate::common::{Color, Rect};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::shared_game_state::SharedGameState;
use crate::graphics::font::Font;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::input::touch_controls::{TouchButton, TouchGeometry};

const TOP_BUTTON_WIDTH: isize = 64;
const TOP_BUTTON_HEIGHT: isize = 20;

/// Full-screen editor where the on-screen controls are dragged into place.
pub struct TouchLayoutEditor {
    /// The button being dragged: touch id, button, and the offset from the finger to the button's centre.
    dragging: Option<(u64, TouchButton, (f32, f32))>,
}

impl TouchLayoutEditor {
    pub fn new() -> TouchLayoutEditor {
        TouchLayoutEditor { dragging: None }
    }

    /// "Reset" and "Done", side by side at the top centre of the safe area.
    fn top_buttons(geometry: &TouchGeometry) -> (Rect<isize>, Rect<isize>) {
        let (left, top, right, _) = geometry.area();
        let center = ((left + right) / 2.0) as isize;
        let y = top as isize + 6;

        (
            Rect::new_size(center - TOP_BUTTON_WIDTH - 4, y, TOP_BUTTON_WIDTH, TOP_BUTTON_HEIGHT),
            Rect::new_size(center + 4, y, TOP_BUTTON_WIDTH, TOP_BUTTON_HEIGHT),
        )
    }

    fn label(state: &SharedGameState, button: TouchButton) -> &str {
        state.loc.t(match button {
            TouchButton::DPad => "menus.controls_menu.touch_menu.labels.dpad",
            TouchButton::Jump => "menus.controls_menu.touch_menu.labels.jump",
            TouchButton::Shoot => "menus.controls_menu.touch_menu.labels.shoot",
            TouchButton::Inventory => "menus.controls_menu.touch_menu.labels.inventory",
            TouchButton::Pause => "menus.controls_menu.touch_menu.labels.pause",
        })
    }

    /// Returns true once the player is done editing.
    pub fn tick(
        &mut self,
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult<bool> {
        let geometry = TouchGeometry::new(ctx, state.canvas_size, state.scale, &state.settings.touch_layout);
        let (reset_rect, done_rect) = Self::top_buttons(&geometry);

        if controller.trigger_back() || state.touch_controls.consume_tap_in(done_rect) {
            state.sound_manager.play_sfx(18);
            self.dragging = None;
            state.settings.save(ctx)?;
            return Ok(true);
        }

        if state.touch_controls.consume_tap_in(reset_rect) {
            state.sound_manager.play_sfx(1);
            state.settings.touch_layout.reset_positions();
            state.settings.save(ctx)?;
        }

        match self.dragging {
            Some((id, button, (offset_x, offset_y))) => {
                if let Some(point) = state.touch_controls.points.iter().find(|p| p.touch_id == id) {
                    let position = geometry.to_fraction(
                        button,
                        point.position.0 as f32 + offset_x,
                        point.position.1 as f32 + offset_y,
                    );
                    state.settings.touch_layout.set_position(button, Some(position));
                } else {
                    self.dragging = None;
                    state.settings.save(ctx)?;
                }
            }
            None => {
                let layout = &state.settings.touch_layout;
                // Small buttons first, so one dropped on the d-pad can still be picked up.
                let grabbed = state.touch_controls.points.iter().find_map(|point| {
                    TouchButton::ALL.iter().rev().find(|&&button| point.is_in(geometry.rect(layout, button))).map(
                        |&button| {
                            let (x, y) = geometry.center(layout, button);
                            (point.touch_id, button, (x - point.position.0 as f32, y - point.position.1 as f32))
                        },
                    )
                });

                if grabbed.is_some() {
                    state.sound_manager.play_sfx(1);
                    self.dragging = grabbed;
                }
            }
        }

        Ok(false)
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let geometry = TouchGeometry::new(ctx, state.canvas_size, state.scale, &state.settings.touch_layout);
        let scale = state.scale;

        graphics::draw_rect(
            ctx,
            Rect::new_size(0, 0, (state.canvas_size.0 * scale) as isize, (state.canvas_size.1 * scale) as isize),
            Color::new(0.0, 0.0, 0.0, 0.55),
        )?;

        let highlight = self.dragging.map(|(_, button, _)| button);
        state.touch_controls.draw_buttons(
            &geometry,
            &state.settings.touch_layout,
            highlight,
            &state.constants,
            &mut state.texture_set,
            ctx,
        )?;

        // Name each control: below it, else above, else beside it, wherever no other button is.
        let rects: Vec<Rect<isize>> =
            TouchButton::ALL.iter().map(|&button| geometry.rect(&state.settings.touch_layout, button)).collect();
        let line_height = state.font.line_height();

        for (button, rect) in TouchButton::ALL.iter().zip(&rects) {
            let label = Self::label(state, *button).to_owned();
            let width = state.font.builder().compute_width(&label);
            let (center_x, center_y) = ((rect.left + rect.right) as f32 / 2.0, (rect.top + rect.bottom) as f32 / 2.0);

            let candidates = [
                (center_x - width / 2.0, rect.bottom as f32 + 2.0),
                (center_x - width / 2.0, rect.top as f32 - line_height - 2.0),
                (rect.left as f32 - width - 4.0, center_y - line_height / 2.0),
                (rect.right as f32 + 4.0, center_y - line_height / 2.0),
            ];
            let fits = |&(x, y): &(f32, f32)| {
                x >= 0.0
                    && y >= 0.0
                    && x + width <= state.canvas_size.0
                    && y + line_height <= state.canvas_size.1
                    && !rects.iter().any(|r| {
                        x < r.right as f32 && x + width > r.left as f32 && y < r.bottom as f32 && y + line_height > r.top as f32
                    })
            };
            let (x, y) = candidates.iter().copied().find(fits).unwrap_or(candidates[0]);

            state.font.builder().position(x, y).shadow(true).draw(&label, ctx, &state.constants, &mut state.texture_set)?;
        }

        let (reset_rect, done_rect) = Self::top_buttons(&geometry);
        let hint = state.loc.t("menus.controls_menu.touch_menu.editor_hint").to_owned();
        let hint_width = state.font.builder().compute_width(&hint);
        state
            .font
            .builder()
            .position((state.canvas_size.0 - hint_width) / 2.0, done_rect.bottom as f32 + 6.0)
            .shadow(true)
            .draw(&hint, ctx, &state.constants, &mut state.texture_set)?;

        for (rect, key) in [
            (reset_rect, "menus.controls_menu.touch_menu.editor_reset"),
            (done_rect, "menus.controls_menu.touch_menu.editor_done"),
        ] {
            let screen_rect = Rect::new_size(
                (rect.left as f32 * scale) as isize,
                (rect.top as f32 * scale) as isize,
                (rect.width() as f32 * scale) as isize,
                (rect.height() as f32 * scale) as isize,
            );
            graphics::draw_rect(ctx, screen_rect, Color::new(0.05, 0.06, 0.12, 0.9))?;
            graphics::draw_outline_rect(ctx, screen_rect, scale.max(1.0) as usize, Color::new(1.0, 1.0, 1.0, 0.85))?;

            let text = state.loc.t(key).to_owned();
            let width = state.font.builder().compute_width(&text);
            state
                .font
                .builder()
                .position((rect.left + rect.right) as f32 / 2.0 - width / 2.0, rect.top as f32 + 5.0)
                .draw(&text, ctx, &state.constants, &mut state.texture_set)?;
        }

        Ok(())
    }
}
