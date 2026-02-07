//! Description of mouse coordinates and state of buttons.

/// Mouse position.
pub type MousePosition = (i32, i32);

/// MouseButton.
pub type MouseButton = usize;

/// Scroll delta represents scroll wheel movement.
/// Values can be positive or negative indicating direction.
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct ScrollDelta {
    /// Vertical scroll delta (positive = up, negative = down)
    pub vertical: i32,
    /// Horizontal scroll delta (positive = right, negative = left)
    pub horizontal: i32,
}

/// Represents a mouse scroll event with direction.
#[derive(Debug, PartialEq, Clone)]
pub enum MouseScrollEvent {
    /// Vertical scroll up
    VerticalUp,
    /// Vertical scroll down
    VerticalDown,
    /// Horizontal scroll right
    HorizontalRight,
    /// Horizontal scroll left
    HorizontalLeft,
}

#[derive(Debug, PartialEq, Default, Clone)]
/// A simple structure containing the current mouse coordinates and the
/// state of each mouse button that we can query. Currently, Windows and
/// Linux provide nice ways to query five mouse buttons. Since button
/// numbers are 1-based, `button_pressed[0]` is assumed to be false and
/// have no meaning.
pub struct MouseState {
    /// Coordinates in pixel.
    pub coords: MousePosition,
    /// State of each mouse button.
    pub button_pressed: Vec<bool>,
    /// Scroll wheel delta since last query.
    pub scroll_delta: ScrollDelta,
}
