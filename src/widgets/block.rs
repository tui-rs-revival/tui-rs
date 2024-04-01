//! Elements related to the `Block` base widget.
//!
//! This holds everything needed to display and configure a [`Block`].
//!
//! In its simplest form, a `Block` is a [border](Borders) around another widget. It can have a
//! [title](Block::title) and [padding](Block::padding).

use itertools::Itertools;
use strum::{Display, EnumString};

use crate::{prelude::*, symbols::border, widgets::Borders};

mod padding;

pub use padding::Padding;

/// Base widget to be used to display a box border around all [upper level ones](crate::widgets).
///
/// The borders can be configured with [`Block::borders`] and others. A block can have multiple
/// titles using [`Block::title`]. It can also be [styled](Block::style) and
/// [padded](Block::padding).
///
/// You can call the title methods multiple times to add multiple titles. Each title will be
/// rendered with a single space separating titles that are in the same position or alignment. When
/// both centered and non-centered titles are rendered, the centered space is calculated based on
/// the full width of the block, rather than the leftover width.
///
/// Titles are not rendered in the corners of the block unless there is no border on that edge.
/// If the block is too small and multiple titles overlap, the border may get cut off at a corner.
///
/// ```plain
/// ┌With at least a left border───
///
/// Without left border───
/// ```
///
/// # Examples
///
/// ```
/// use ratatui::{prelude::*, widgets::*};
///
/// Block::default()
///     .title("Block")
///     .borders(Borders::LEFT | Borders::RIGHT)
///     .border_style(Style::default().fg(Color::White))
///     .border_type(BorderType::Rounded)
///     .style(Style::default().bg(Color::Black));
/// ```
///
/// You may also use multiple titles like in the following:
/// ```
/// use ratatui::{
///     prelude::*,
///     widgets::{block::*, *},
/// };
///
/// Block::default()
///     .title("Title 1")
///     .title_(Line::raw("Title 2").position(Position::Bottom));
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Block<'a> {
    /// List of titles
    top_titles: Vec<Line<'a>>,
    bottom_titles: Vec<Line<'a>>,
    /// The style to be patched to all titles of the block
    titles_style: Style,
    /// The default alignment of the titles that don't have one
    titles_alignment: Alignment,
    /// Visible borders
    borders: Borders,
    /// Border style
    border_style: Style,
    /// The symbols used to render the border. The default is plain lines but one can choose to
    /// have rounded or doubled lines instead or a custom set of symbols
    border_set: border::Set,
    /// Widget style
    style: Style,
    /// Block padding
    padding: Padding,
}

/// The type of border of a [`Block`].
///
/// See the [`borders`](Block::borders) method of `Block` to configure its borders.
#[derive(Debug, Default, Display, EnumString, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BorderType {
    /// A plain, simple border.
    ///
    /// This is the default
    ///
    /// # Example
    ///
    /// ```plain
    /// ┌───────┐
    /// │       │
    /// └───────┘
    /// ```
    #[default]
    Plain,
    /// A plain border with rounded corners.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╭───────╮
    /// │       │
    /// ╰───────╯
    /// ```
    Rounded,
    /// A doubled border.
    ///
    /// Note this uses one character that draws two lines.
    ///
    /// # Example
    ///
    /// ```plain
    /// ╔═══════╗
    /// ║       ║
    /// ╚═══════╝
    /// ```
    Double,
    /// A thick border.
    ///
    /// # Example
    ///
    /// ```plain
    /// ┏━━━━━━━┓
    /// ┃       ┃
    /// ┗━━━━━━━┛
    /// ```
    Thick,
    /// A border with a single line on the inside of a half block.
    ///
    /// # Example
    ///
    /// ```plain
    /// ▗▄▄▄▄▄▄▄▖
    /// ▐       ▌
    /// ▐       ▌
    /// ▝▀▀▀▀▀▀▀▘
    QuadrantInside,

    /// A border with a single line on the outside of a half block.
    ///
    /// # Example
    ///
    /// ```plain
    /// ▛▀▀▀▀▀▀▀▜
    /// ▌       ▐
    /// ▌       ▐
    /// ▙▄▄▄▄▄▄▄▟
    QuadrantOutside,
}

impl<'a> Block<'a> {
    /// Creates a new block with no [`Borders`] or [`Padding`].
    pub const fn new() -> Self {
        Self {
            top_titles: Vec::new(),
            bottom_titles: Vec::new(),
            titles_style: Style::new(),
            titles_alignment: Alignment::Left,
            borders: Borders::NONE,
            border_style: Style::new(),
            border_set: BorderType::Plain.to_border_set(),
            style: Style::new(),
            padding: Padding::zero(),
        }
    }

    /// Create a new block with [all borders](Borders::ALL) shown
    pub const fn bordered() -> Self {
        let mut block = Self::new();
        block.borders = Borders::ALL;
        block
    }

    /// Adds a title to the block.
    ///
    /// The `title` function allows you to add a title to the block. You can call this function
    /// multiple times to add multiple titles.
    ///
    /// Each title will be rendered with a single space separating titles that are in the same
    /// position or alignment. When both centered and non-centered titles are rendered, the centered
    /// space is calculated based on the full width of the block, rather than the leftover width.
    ///
    /// You can provide any type that can be converted into [`Line`] including: strings, string
    /// slices (`&str`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// By default, the titles will avoid being rendered in the corners of the block but will align
    /// against the left or right edge of the block if there is no border on that edge.
    /// The following demonstrates this behavior, notice the second title is one character off to
    /// the left.
    ///
    /// ```plain
    /// ┌With at least a left border───
    ///
    /// Without left border───
    /// ```
    ///
    /// Note: If the block is too small and multiple titles overlap, the border might get cut off at
    /// a corner.
    ///
    /// # Example
    ///
    /// The following example demonstrates:
    /// - Default title alignment
    /// - Multiple titles (notice "Center" is centered according to the full with of the block, not
    /// the leftover space)
    /// - Two titles with the same alignment (notice the left titles are separated)
    /// ```
    /// use ratatui::{
    ///     prelude::*,
    ///     widgets::{block::*, *},
    /// };
    ///
    /// Block::default()
    ///     .title("Title") // By default in the top left corner
    ///     .title(Line::raw("Left").alignment(Alignment::Left)) // also on the left
    ///     .title(Line::raw("Right").alignment(Alignment::Right))
    ///     .title(Line::raw("Center").alignment(Alignment::Center));
    /// // Renders
    /// // ┌Title─Left────Center─────────Right┐
    /// ```
    ///
    /// # See also
    ///
    /// Titles attached to a block can have default behaviors. See
    /// - [`Block::title_style`]
    /// - [`Block::title_alignment`]
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title<T>(self, title: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        self.top_title(title)
    }

    /// Adds a title to the top of the block.
    ///
    /// You can provide any type that can be converted into [`Line`] including: strings, string
    /// slices (`&str`), borrowed strings (`Cow<str>`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::{ prelude::*, widgets::* };
    /// Block::bordered()
    ///     .title_top("Left1") // By default in the top left corner
    ///     .title_top(Line::from("Left2").left_aligned())
    ///     .title_top(Line::from("Right").right_aligned())
    ///     .title_top(Line::from("Center").centered());
    ///
    /// // Renders
    /// // ┌Left1─Left2───Center─────────Right┐
    /// // │                                  │
    /// // └──────────────────────────────────┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn top_title<T: Into<Line<'a>>>(mut self, title: T) -> Self {
        self.top_titles.push(title.into());
        self
    }

    /// Adds a title to the bottom of the block.
    ///
    /// You can provide any type that can be converted into [`Line`] including: strings, string
    /// slices (`&str`), borrowed strings (`Cow<str>`), [spans](crate::text::Span), or vectors of
    /// [spans](crate::text::Span) (`Vec<Span>`).
    ///
    /// # Example
    ///
    /// ```
    /// # use ratatui::{ prelude::*, widgets::* };
    /// Block::bordered()
    ///     .title_bottom("Left1") // By default in the top left corner
    ///     .title_bottom(Line::from("Left2").left_aligned())
    ///     .title_bottom(Line::from("Right").right_aligned())
    ///     .title_bottom(Line::from("Center").centered());
    ///
    /// // Renders
    /// // ┌──────────────────────────────────┐
    /// // │                                  │
    /// // └Left1─Left2───Center─────────Right┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn bottom_title<T: Into<Line<'a>>>(mut self, title: T) -> Self {
        self.bottom_titles.push(title.into());
        self
    }

    /// Applies the style to all titles.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// If a title already has a style, the title's style will add on top of this one.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn title_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.titles_style = style.into();
        self
    }

    /// Sets the default [`Alignment`] for all block titles.
    ///
    /// Titles that explicitly set an [`Alignment`] will ignore this.
    ///
    /// # Example
    ///
    /// This example aligns all titles in the center except the "right" title which explicitly sets
    /// [`Alignment::Right`].
    /// ```
    /// use ratatui::{
    ///     prelude::*,
    ///     widgets::{block::*, *},
    /// };
    ///
    /// Block::default()
    ///     // This title won't be aligned in the center
    ///     .title(Line::raw("right").alignment(Alignment::Right))
    ///     .title("foo")
    ///     .title("bar")
    ///     .title_alignment(Alignment::Center);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn title_alignment(mut self, alignment: Alignment) -> Self {
        self.titles_alignment = alignment;
        self
    }

    /// Defines the style of the borders.
    ///
    /// If a [`Block::style`] is defined, `border_style` will be applied on top of it.
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// # Example
    ///
    /// This example shows a `Block` with blue borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .border_style(Style::new().blue());
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn border_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.border_style = style.into();
        self
    }

    /// Defines the block style.
    ///
    /// This is the most generic [`Style`] a block can receive, it will be merged with any other
    /// more specific style. Elements can be styled further with [`Block::title_style`] and
    /// [`Block::border_style`].
    ///
    /// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`], [`Color`], or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This will also apply to the widget inside that block, unless the inner widget is styled.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Defines which borders to display.
    ///
    /// [`Borders`] can also be styled with [`Block::border_style`] and [`Block::border_type`].
    ///
    /// # Examples
    ///
    /// Simply show all borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().borders(Borders::ALL);
    /// ```
    ///
    /// Display left and right borders.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().borders(Borders::LEFT | Borders::RIGHT);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn borders(mut self, flag: Borders) -> Self {
        self.borders = flag;
        self
    }

    /// Sets the symbols used to display the border (e.g. single line, double line, thick or
    /// rounded borders).
    ///
    /// Setting this overwrites any custom [`border_set`](Block::border_set) that was set.
    ///
    /// See [`BorderType`] for the full list of available symbols.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .title("Block")
    ///     .borders(Borders::ALL)
    ///     .border_type(BorderType::Rounded);
    /// // Renders
    /// // ╭Block╮
    /// // │     │
    /// // ╰─────╯
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_set = border_type.to_border_set();
        self
    }

    /// Sets the symbols used to display the border as a [`crate::symbols::border::Set`].
    ///
    /// Setting this overwrites any [`border_type`](Block::border_type) that was set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default().title("Block").borders(Borders::ALL).border_set(symbols::border::DOUBLE);
    /// // Renders
    /// // ╔Block╗
    /// // ║     ║
    /// // ╚═════╝
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn border_set(mut self, border_set: border::Set) -> Self {
        self.border_set = border_set;
        self
    }

    /// Compute the inner area of a block based on its border visibility rules.
    ///
    /// # Examples
    ///
    /// Draw a block nested within another block
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// # fn render_nested_block(frame: &mut Frame) {
    /// let outer_block = Block::default().title("Outer").borders(Borders::ALL);
    /// let inner_block = Block::default().title("Inner").borders(Borders::ALL);
    ///
    /// let outer_area = frame.size();
    /// let inner_area = outer_block.inner(outer_area);
    ///
    /// frame.render_widget(outer_block, outer_area);
    /// frame.render_widget(inner_block, inner_area);
    /// # }
    /// // Renders
    /// // ┌Outer────────┐
    /// // │┌Inner──────┐│
    /// // ││           ││
    /// // │└───────────┘│
    /// // └─────────────┘
    /// ```
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::TOP) || !self.top_titles.is_empty() {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::BOTTOM) || !self.bottom_titles.is_empty() {
            inner.height = inner.height.saturating_sub(1);
        }

        inner.x = inner.x.saturating_add(self.padding.left);
        inner.y = inner.y.saturating_add(self.padding.top);

        inner.width = inner
            .width
            .saturating_sub(self.padding.left + self.padding.right);
        inner.height = inner
            .height
            .saturating_sub(self.padding.top + self.padding.bottom);

        inner
    }

    /// Defines the padding inside a `Block`.
    ///
    /// See [`Padding`] for more information.
    ///
    /// # Examples
    ///
    /// This renders a `Block` with no padding (the default).
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .padding(Padding::zero());
    /// // Renders
    /// // ┌───────┐
    /// // │content│
    /// // └───────┘
    /// ```
    ///
    /// This example shows a `Block` with padding left and right ([`Padding::horizontal`]).
    /// Notice the two spaces before and after the content.
    /// ```
    /// # use ratatui::{prelude::*, widgets::*};
    /// Block::default()
    ///     .borders(Borders::ALL)
    ///     .padding(Padding::horizontal(2));
    /// // Renders
    /// // ┌───────────┐
    /// // │  content  │
    /// // └───────────┘
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
}

impl BorderType {
    /// Convert this `BorderType` into the corresponding [`Set`](border::Set) of border symbols.
    pub const fn border_symbols(border_type: Self) -> border::Set {
        match border_type {
            Self::Plain => border::PLAIN,
            Self::Rounded => border::ROUNDED,
            Self::Double => border::DOUBLE,
            Self::Thick => border::THICK,
            Self::QuadrantInside => border::QUADRANT_INSIDE,
            Self::QuadrantOutside => border::QUADRANT_OUTSIDE,
        }
    }

    /// Convert this `BorderType` into the corresponding [`Set`](border::Set) of border symbols.
    pub const fn to_border_set(self) -> border::Set {
        Self::border_symbols(self)
    }
}

impl Widget for Block<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

impl WidgetRef for Block<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let area = area.intersection(buf.area);
        if area.is_empty() {
            return;
        }
        buf.set_style(area, self.style);
        self.render_borders(area, buf);
        self.render_titles(area, buf);
    }
}

impl Block<'_> {
    fn render_borders(&self, area: Rect, buf: &mut Buffer) {
        self.render_left_side(area, buf);
        self.render_top_side(area, buf);
        self.render_right_side(area, buf);
        self.render_bottom_side(area, buf);

        self.render_bottom_right_corner(buf, area);
        self.render_top_right_corner(buf, area);
        self.render_bottom_left_corner(buf, area);
        self.render_top_left_corner(buf, area);
    }

    fn render_titles(&self, area: Rect, buf: &mut Buffer) {
        let title_areas = self.title_areas(area);

        let right_titles = self.filtered_titles(Alignment::Right);
        Self::render_right_titles(
            &right_titles.0.collect_vec(),
            title_areas.0,
            buf,
            self.style,
        );
        Self::render_right_titles(
            &right_titles.1.collect_vec(),
            title_areas.1,
            buf,
            self.style,
        );

        let center_titles = self.filtered_titles(Alignment::Center);
        Self::render_center_titles(
            &center_titles.0.collect_vec(),
            title_areas.0,
            buf,
            self.style,
        );
        Self::render_center_titles(
            &center_titles.1.collect_vec(),
            title_areas.1,
            buf,
            self.style,
        );

        let left_titles = self.filtered_titles(Alignment::Left);
        Self::render_left_titles(&left_titles.0.collect_vec(), title_areas.0, buf, self.style);
        Self::render_left_titles(&left_titles.1.collect_vec(), title_areas.1, buf, self.style);
    }
    fn render_left_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(self.border_set.vertical_left)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_top_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(self.border_set.horizontal_top)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_right_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::RIGHT) {
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.vertical_right)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_bottom_side(&self, area: Rect, buf: &mut Buffer) {
        if self.borders.contains(Borders::BOTTOM) {
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(self.border_set.horizontal_bottom)
                    .set_style(self.border_style);
            }
        }
    }

    fn render_bottom_right_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(area.right() - 1, area.bottom() - 1)
                .set_symbol(self.border_set.bottom_right)
                .set_style(self.border_style);
        }
    }

    fn render_top_right_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(area.right() - 1, area.top())
                .set_symbol(self.border_set.top_right)
                .set_style(self.border_style);
        }
    }

    fn render_bottom_left_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(area.left(), area.bottom() - 1)
                .set_symbol(self.border_set.bottom_left)
                .set_style(self.border_style);
        }
    }

    fn render_top_left_corner(&self, buf: &mut Buffer, area: Rect) {
        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(area.left(), area.top())
                .set_symbol(self.border_set.top_left)
                .set_style(self.border_style);
        }
    }

    /// Render titles aligned to the right of the block
    ///
    /// Currently (due to the way lines are truncated), the right side of the leftmost title will
    /// be cut off if the block is too small to fit all titles. This is not ideal and should be
    /// the left side of that leftmost that is cut off. This is due to the line being truncated
    /// incorrectly. See <https://github.com/ratatui-org/ratatui/issues/932>
    #[allow(clippy::similar_names)]
    fn render_right_titles(
        titles: &[&Line],
        mut titles_area: Rect,
        buf: &mut Buffer,
        style: Style,
    ) {
        // render titles in reverse order to align them to the right
        for title in titles.iter().rev() {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.width() as u16;
            let title_area = Rect {
                x: titles_area
                    .right()
                    .saturating_sub(title_width)
                    .max(titles_area.left()),
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, style);
            title.render_ref(title_area, buf);

            // bump the width of the titles area to the left
            titles_area.width = titles_area
                .width
                .saturating_sub(title_width)
                .saturating_sub(1); // space between titles
        }
    }

    /// Render titles in the center of the block
    ///
    /// Currently this method aligns the titles to the left inside a centered area. This is not
    /// ideal and should be fixed in the future to align the titles to the center of the block and
    /// truncate both sides of the titles if the block is too small to fit all titles.
    #[allow(clippy::similar_names)]
    fn render_center_titles(titles: &[&Line], titles_area: Rect, buf: &mut Buffer, style: Style) {
        let total_width = titles
            .iter()
            .map(|title| title.width() as u16 + 1) // space between titles
            .sum::<u16>()
            .saturating_sub(1); // no space for the last title
        let mut titles_area = Rect {
            x: titles_area.left() + (titles_area.width.saturating_sub(total_width) / 2),
            ..titles_area
        };
        for title in titles {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.width() as u16;
            let title_area = Rect {
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, style);
            title.render_ref(title_area, buf);

            // bump the titles area to the right and reduce its width
            titles_area.x = titles_area.x.saturating_add(title_width + 1);
            titles_area.width = titles_area.width.saturating_sub(title_width + 1);
        }
    }

    /// Render titles aligned to the left of the block
    #[allow(clippy::similar_names)]
    fn render_left_titles(titles: &[&Line], mut titles_area: Rect, buf: &mut Buffer, style: Style) {
        for title in titles {
            if titles_area.is_empty() {
                break;
            }
            let title_width = title.width() as u16;
            let title_area = Rect {
                width: title_width.min(titles_area.width),
                ..titles_area
            };
            buf.set_style(title_area, style);
            title.render_ref(title_area, buf);

            // bump the titles area to the right and reduce its width
            titles_area.x = titles_area.x.saturating_add(title_width + 1);
            titles_area.width = titles_area.width.saturating_sub(title_width + 1);
        }
    }
    /// An iterator over the titles that match the position and alignment
    fn filtered_titles(
        &self,
        alignment: Alignment,
    ) -> (
        impl DoubleEndedIterator<Item = &Line>,
        impl DoubleEndedIterator<Item = &Line>,
    ) {
        (
            self.top_titles
                .iter()
                .filter(move |title| title.alignment.unwrap_or(self.titles_alignment) == alignment),
            self.bottom_titles
                .iter()
                .filter(move |title| title.alignment.unwrap_or(self.titles_alignment) == alignment),
        )
    }

    /// An area that is one line tall and spans the width of the block excluding the borders and
    /// is positioned at the top or bottom of the block.
    fn title_areas(&self, area: Rect) -> (Rect, Rect) {
        (
            self.title_area(area.top(), area),
            self.title_area(area.bottom() - 1, area),
        )
    }

    fn title_area(&self, y: u16, area: Rect) -> Rect {
        let left_border = u16::from(self.borders.contains(Borders::LEFT));
        let right_border = u16::from(self.borders.contains(Borders::RIGHT));
        Rect {
            x: area.left() + left_border,
            y,
            width: area
                .width
                .saturating_sub(left_border)
                .saturating_sub(right_border),
            height: 1,
        }
    }
}

/// An extension trait for [`Block`] that provides some convenience methods.
///
/// This is implemented for [`Option<Block>`](Option) to simplify the common case of having a
/// widget with an optional block.
pub trait BlockExt {
    /// Return the inner area of the block if it is `Some`. Otherwise, returns `area`.
    ///
    /// This is a useful convenience method for widgets that have an `Option<Block>` field
    fn inner_if_some(&self, area: Rect) -> Rect;
}

impl BlockExt for Option<Block<'_>> {
    fn inner_if_some(&self, area: Rect) -> Rect {
        self.as_ref().map_or(area, |block| block.inner(area))
    }
}

impl<'a> Styled for Block<'a> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(self, style: S) -> Self::Item {
        self.style(style)
    }
}

#[cfg(test)]
mod tests {
    use strum::ParseError;

    use super::*;
    use crate::assert_buffer_eq;

    #[test]
    fn create_with_all_borders() {
        let block = Block::bordered();
        assert_eq!(block.borders, Borders::all());
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn inner_takes_into_account_the_borders() {
        // No borders
        assert_eq!(
            Block::default().inner(Rect::default()),
            Rect::new(0, 0, 0, 0),
            "no borders, width=0, height=0"
        );
        assert_eq!(
            Block::default().inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 1, 1),
            "no borders, width=1, height=1"
        );

        // Left border
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 0, 0, 1),
            "left, width=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(1, 0, 0, 1),
            "left, width=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::LEFT)
                .inner(Rect::new(0, 0, 2, 1)),
            Rect::new(1, 0, 1, 1),
            "left, width=2"
        );

        // Top border
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 0)),
            Rect::new(0, 0, 1, 0),
            "top, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 1, 1, 0),
            "top, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::TOP)
                .inner(Rect::new(0, 0, 1, 2)),
            Rect::new(0, 1, 1, 1),
            "top, height=2"
        );

        // Right border
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 0, 0, 1),
            "right, width=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 0, 1),
            "right, width=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::RIGHT)
                .inner(Rect::new(0, 0, 2, 1)),
            Rect::new(0, 0, 1, 1),
            "right, width=2"
        );

        // Bottom border
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 0)),
            Rect::new(0, 0, 1, 0),
            "bottom, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(0, 0, 1, 0),
            "bottom, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::BOTTOM)
                .inner(Rect::new(0, 0, 1, 2)),
            Rect::new(0, 0, 1, 1),
            "bottom, height=2"
        );

        // All borders
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::default()),
            Rect::new(0, 0, 0, 0),
            "all borders, width=0, height=0"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 1, 1)),
            Rect::new(1, 1, 0, 0),
            "all borders, width=1, height=1"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 2, 2)),
            Rect::new(1, 1, 0, 0),
            "all borders, width=2, height=2"
        );
        assert_eq!(
            Block::default()
                .borders(Borders::ALL)
                .inner(Rect::new(0, 0, 3, 3)),
            Rect::new(1, 1, 1, 1),
            "all borders, width=3, height=3"
        );
    }

    #[test]
    fn inner_takes_into_account_the_title() {
        assert_eq!(
            Block::default().title("Test").inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
        assert_eq!(
            Block::default()
                .title(Line::raw("Test").centered())
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
        assert_eq!(
            Block::default()
                .title(Line::raw("Test").right_aligned())
                .inner(Rect::new(0, 0, 0, 1)),
            Rect::new(0, 1, 0, 0),
        );
    }

    #[test]
    fn inner_takes_into_account_border_and_title() {
        let test_rect = Rect::new(0, 0, 0, 2);

        let top_top = Block::default()
            .title(Line::raw("Test"))
            .borders(Borders::TOP);
        assert_eq!(top_top.inner(test_rect), Rect::new(0, 1, 0, 1));

        let top_bot = Block::default()
            .title(Line::raw("Test"))
            .borders(Borders::BOTTOM);
        assert_eq!(top_bot.inner(test_rect), Rect::new(0, 1, 0, 0));

        let bot_top = Block::default()
            .bottom_title(Line::raw("Test"))
            .borders(Borders::TOP);
        assert_eq!(bot_top.inner(test_rect), Rect::new(0, 1, 0, 0));

        let bot_bot = Block::default()
            .bottom_title(Line::raw("Test"))
            .borders(Borders::BOTTOM);
        assert_eq!(bot_bot.inner(test_rect), Rect::new(0, 0, 0, 1));
    }

    #[test]
    const fn border_type_can_be_const() {
        const _PLAIN: border::Set = BorderType::border_symbols(BorderType::Plain);
    }

    #[test]
    fn block_new() {
        assert_eq!(
            Block::new(),
            Block {
                top_titles: Vec::new(),
                bottom_titles: Vec::new(),
                titles_style: Style::new(),
                titles_alignment: Alignment::Left,
                borders: Borders::NONE,
                border_style: Style::new(),
                border_set: BorderType::Plain.to_border_set(),
                style: Style::new(),
                padding: Padding::zero(),
            }
        );
    }

    #[test]
    const fn block_can_be_const() {
        const _DEFAULT_STYLE: Style = Style::new();
        const _DEFAULT_PADDING: Padding = Padding::uniform(1);
        const _DEFAULT_BLOCK: Block = Block::new()
            // the following methods are no longer const because they use Into<Style>
            // .style(_DEFAULT_STYLE)           // no longer const
            // .border_style(_DEFAULT_STYLE)    // no longer const
            // .title_style(_DEFAULT_STYLE)     // no longer const
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .padding(_DEFAULT_PADDING);
    }

    /// This test ensures that we have some coverage on the [`Style::from()`] implementations
    #[test]
    fn block_style() {
        // nominal style
        let block = Block::default().style(Style::new().red());
        assert_eq!(block.style, Style::new().red());

        // auto-convert from Color
        let block = Block::default().style(Color::Red);
        assert_eq!(block.style, Style::new().red());

        // auto-convert from (Color, Color)
        let block = Block::default().style((Color::Red, Color::Blue));
        assert_eq!(block.style, Style::new().red().on_blue());

        // auto-convert from Modifier
        let block = Block::default().style(Modifier::BOLD | Modifier::ITALIC);
        assert_eq!(block.style, Style::new().bold().italic());

        // auto-convert from (Modifier, Modifier)
        let block = Block::default().style((Modifier::BOLD | Modifier::ITALIC, Modifier::DIM));
        assert_eq!(block.style, Style::new().bold().italic().not_dim());

        // auto-convert from (Color, Modifier)
        let block = Block::default().style((Color::Red, Modifier::BOLD));
        assert_eq!(block.style, Style::new().red().bold());

        // auto-convert from (Color, Color, Modifier)
        let block = Block::default().style((Color::Red, Color::Blue, Modifier::BOLD));
        assert_eq!(block.style, Style::new().red().on_blue().bold());

        // auto-convert from (Color, Color, Modifier, Modifier)
        let block = Block::default().style((
            Color::Red,
            Color::Blue,
            Modifier::BOLD | Modifier::ITALIC,
            Modifier::DIM,
        ));
        assert_eq!(
            block.style,
            Style::new().red().on_blue().bold().italic().not_dim()
        );
    }

    #[test]
    fn can_be_stylized() {
        let block = Block::default().black().on_white().bold().not_dim();
        assert_eq!(
            block.style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        );
    }

    #[test]
    fn title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::bordered()
            .top_title(Line::raw("A").left_aligned())
            .top_title(Line::raw("B").centered())
            .top_title(Line::raw("C").right_aligned())
            .bottom_title(Line::raw("D").left_aligned())
            .bottom_title(Line::raw("E").centered())
            .bottom_title(Line::raw("F").right_aligned())
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌A─────B─────C┐",
                "│             │",
                "└D─────E─────F┘",
            ])
        );
    }

    #[test]
    fn title_alignment() {
        let tests = vec![
            (Alignment::Left, "test    "),
            (Alignment::Center, "  test  "),
            (Alignment::Right, "    test"),
        ];
        for (alignment, expected) in tests {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 1));
            Block::default()
                .title("test")
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);
            assert_buffer_eq!(buffer, Buffer::with_lines(vec![expected]));
        }
    }

    #[test]
    fn title_alignment_overrides_block_title_alignment() {
        let tests = vec![
            (Alignment::Right, Alignment::Left, "test    "),
            (Alignment::Left, Alignment::Center, "  test  "),
            (Alignment::Center, Alignment::Right, "    test"),
        ];
        for (block_title_alignment, alignment, expected) in tests {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 8, 1));
            Block::default()
                .title(Line::raw("test").alignment(alignment))
                .title_alignment(block_title_alignment)
                .render(buffer.area, &mut buffer);
            assert_buffer_eq!(buffer, Buffer::with_lines(vec![expected]));
        }
    }

    /// This is a regression test for bug <https://github.com/ratatui-org/ratatui/issues/929>
    #[test]
    fn render_right_aligned_empty_title() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .title("")
            .title_alignment(Alignment::Right)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "               ",
                "               ",
                "               ",
            ])
        );
    }

    #[test]
    fn title_position() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 2));
        Block::default()
            .bottom_title("test")
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(buffer, Buffer::with_lines(vec!["    ", "test"]));
    }

    #[test]
    fn title_content_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title("test".yellow())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn block_title_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title("test")
                .title_style(Style::new().yellow())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn title_style_overrides_block_title_style() {
        for alignment in [Alignment::Left, Alignment::Center, Alignment::Right] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 4, 1));
            Block::default()
                .title(Line::raw("test").yellow())
                .title_style(Style::new().green().on_red())
                .title_alignment(alignment)
                .render(buffer.area, &mut buffer);

            let mut expected_buffer = Buffer::with_lines(vec!["test"]);
            expected_buffer.set_style(Rect::new(0, 0, 4, 1), Style::new().yellow().on_red());

            assert_buffer_eq!(buffer, expected_buffer);
        }
    }

    #[test]
    fn title_border_style() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .title("test")
            .borders(Borders::ALL)
            .border_style(Style::new().yellow())
            .render(buffer.area, &mut buffer);

        let mut expected_buffer = Buffer::with_lines(vec![
            "┌test─────────┐",
            "│             │",
            "└─────────────┘",
        ]);
        expected_buffer.set_style(Rect::new(0, 0, 15, 3), Style::new().yellow());
        expected_buffer.set_style(Rect::new(1, 1, 13, 1), Style::reset());

        assert_buffer_eq!(buffer, expected_buffer);
    }

    #[test]
    fn border_type_to_string() {
        assert_eq!(format!("{}", BorderType::Plain), "Plain");
        assert_eq!(format!("{}", BorderType::Rounded), "Rounded");
        assert_eq!(format!("{}", BorderType::Double), "Double");
        assert_eq!(format!("{}", BorderType::Thick), "Thick");
    }

    #[test]
    fn border_type_from_str() {
        assert_eq!("Plain".parse(), Ok(BorderType::Plain));
        assert_eq!("Rounded".parse(), Ok(BorderType::Rounded));
        assert_eq!("Double".parse(), Ok(BorderType::Double));
        assert_eq!("Thick".parse(), Ok(BorderType::Thick));
        assert_eq!("".parse::<BorderType>(), Err(ParseError::VariantNotFound));
    }

    #[test]
    fn render_plain_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┌─────────────┐",
                "│             │",
                "└─────────────┘"
            ])
        );
    }

    #[test]
    fn render_rounded_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╭─────────────╮",
                "│             │",
                "╰─────────────╯"
            ])
        );
    }

    #[test]
    fn render_double_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "╔═════════════╗",
                "║             ║",
                "╚═════════════╝"
            ])
        );
    }

    #[test]
    fn render_quadrant_inside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantInside)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▗▄▄▄▄▄▄▄▄▄▄▄▄▄▖",
                "▐             ▌",
                "▝▀▀▀▀▀▀▀▀▀▀▀▀▀▘",
            ])
        );
    }

    #[test]
    fn render_border_quadrant_outside() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::QuadrantOutside)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "▛▀▀▀▀▀▀▀▀▀▀▀▀▀▜",
                "▌             ▐",
                "▙▄▄▄▄▄▄▄▄▄▄▄▄▄▟",
            ])
        );
    }

    #[test]
    fn render_solid_border() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "┏━━━━━━━━━━━━━┓",
                "┃             ┃",
                "┗━━━━━━━━━━━━━┛"
            ])
        );
    }

    #[test]
    fn render_custom_border_set() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 15, 3));
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::Set {
                top_left: "1",
                top_right: "2",
                bottom_left: "3",
                bottom_right: "4",
                vertical_left: "L",
                vertical_right: "R",
                horizontal_top: "T",
                horizontal_bottom: "B",
            })
            .render(buffer.area, &mut buffer);
        assert_buffer_eq!(
            buffer,
            Buffer::with_lines(vec![
                "1TTTTTTTTTTTTT2",
                "L             R",
                "3BBBBBBBBBBBBB4",
            ])
        );
    }
}
