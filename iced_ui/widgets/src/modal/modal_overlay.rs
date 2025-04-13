//! A modal for showing elements as an overlay on top of another.
//!
//! *This API requires the following crate features to be activated: modal*

use deps::*;

use super::style::StyleSheet;
use iced::{
    advanced::{
        layout::{Limits, Node},
        overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Layout, Overlay, Shell,
    },
    alignment, event, keyboard,
    mouse::{self, Cursor},
    touch, Alignment, Border, Color, Element, Event, Rectangle, Shadow, Size, Vector,
};

/// The overlay of the modal.
#[allow(missing_debug_implementations)]
pub struct ModalOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    /// The state of the [`ModalOverlay`](ModalOverlay).
    state: &'b mut Tree,
    /// The content of the [`ModalOverlay`](ModalOverlay).
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    /// The optional message that will be send when the user clicks on the backdrop.
    backdrop: Option<Message>,
    /// The optional message that will be send when the ESC key was pressed.
    esc: Option<Message>,
    /// The style of the [`ModalOverlay`](ModalOverlay).
    style: <Theme as StyleSheet>::Style,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
}

impl<'a, 'b, Message, Theme, Renderer> ModalOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{
    /// Creates a new [`ModalOverlay`](ModalOverlay).
    pub fn new(
        state: &'b mut Tree,
        content: &'b mut Element<'a, Message, Theme, Renderer>,
        backdrop: Option<Message>,
        esc: Option<Message>,
        style: <Theme as StyleSheet>::Style,
        horizontal_alignment: alignment::Horizontal,
        vertical_alignment: alignment::Vertical,
    ) -> Self {
        ModalOverlay {
            state,
            content,
            backdrop,
            esc,
            style,
            horizontal_alignment,
            vertical_alignment,
        }
    }
}

impl<'a, 'b, Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for ModalOverlay<'a, 'b, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet,
{

    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let limits = Limits::new(Size::ZERO, bounds);
        let mut content = self
            .content
            .as_widget()
            .layout(self.state, renderer, &limits);
        let max_size = limits.max();

        content = content.align(
            Alignment::from(self.horizontal_alignment),
            Alignment::from(self.vertical_alignment),
            max_size,
        );

        Node::with_children(max_size, vec![content])
    }

    // fn on_event(
    //     &mut self,
    //     event: Event,
    //     layout: Layout<'_>,
    //     cursor: Cursor,
    //     renderer: &Renderer,
    //     clipboard: &mut dyn Clipboard,
    //     shell: &mut Shell<Message>,
    // ) -> event::Status {
    //     let viewport = layout.bounds();
    //     // TODO clean this up
    //     let esc_status = self
    //         .esc
    //         .as_ref()
    //         .map_or(event::Status::Ignored, |esc| match &event {
    //             Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
    //                 if *key == keyboard::Key::Named(keyboard::key::Named::Escape) {
    //                     shell.publish(esc.to_owned());
    //                     event::Status::Captured
    //                 } else {
    //                     event::Status::Ignored
    //                 }
    //             }
    //             _ => event::Status::Ignored,
    //         });

    //     let backdrop_status = self.backdrop.as_ref().zip(layout.children().next()).map_or(
    //         event::Status::Ignored,
    //         |(backdrop, layout)| match &event {
    //             Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    //             | Event::Touch(touch::Event::FingerPressed { .. }) => {
    //                 if layout
    //                     .bounds()
    //                     .contains(cursor.position().unwrap_or_default())
    //                 {
    //                     event::Status::Ignored
    //                 } else {
    //                     shell.publish(backdrop.to_owned());
    //                     event::Status::Captured
    //                 }
    //             }
    //             _ => event::Status::Ignored,
    //         },
    //     );

    //     match esc_status.merge(backdrop_status) {
    //         event::Status::Ignored => self.content.as_widget_mut().on_event(
    //             self.state,
    //             event,
    //             layout
    //                 .children()
    //                 .next()
    //                 .expect("Native: Layout should have a content layout."),
    //             cursor,
    //             renderer,
    //             clipboard,
    //             shell,
    //             &viewport,
    //         ),
    //         event::Status::Captured => event::Status::Captured,
    //     }
    // }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.state,
            layout
                .children()
                .next()
                .expect("Native: Layout should have a content layout."),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        let bounds = layout.bounds();

        let style_sheet = theme.active(&self.style);

        // Background
        if (bounds.width > 0.) && (bounds.height > 0.) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: (0.0).into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    shadow: Shadow::default(),
                },
                style_sheet.background,
            );
        }

        let content_layout = layout
            .children()
            .next()
            .expect("Native: Layout should have a content layout.");

        // Modal
        self.content.as_widget().draw(
            self.state,
            renderer,
            theme,
            style,
            content_layout,
            cursor,
            &bounds,
        );
    }

    fn overlay<'c>(
        &'c mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        let content_layout = layout.children().next()?;
        self.content.as_widget_mut().overlay(
            self.state,
            content_layout,
            renderer,
            Vector::new(0.0, 0.0),
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        let content_layout = layout
            .children()
            .next()
            .expect("Native: Layout should have a content layout.");

        self.content
            .as_widget()
            .operate(self.state, content_layout, renderer, operation);
    }
}
