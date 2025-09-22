use std::sync::Arc;

use nih_plug_iced::{
    widget::{column, container, text, toggler},
    Element, Length, Renderer, Theme,
};

use crate::{editor::{Message, PrismatineEditor}, util::ProcessMode, PrismatineParams};

impl PrismatineEditor
{



pub fn create_parameter_box(&self
) -> Element<'_, Message, Theme, Renderer> {
    match self.params.prismatine_params.process_mode.value() {
        ProcessMode::Josephson => column![
            text("phase gain").width(Length::Fill).center(),
            container(
                nih_plug_iced::widgets::ParamSlider::new(
                    self.phase_gain_slider_state.clone(),
                    &self.params.prismatine_params.phase_gain,
                )
                .map(Message::ParamUpdate),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),

            text("critical current").width(Length::Fill).center(),

            container(
                nih_plug_iced::widgets::ParamSlider::new(
                    self.I_c_slider_state.clone(),
                    &self.params.prismatine_params.I_c,
                )
                .map(Message::ParamUpdate),
            )
            .width(Length::Fill)
            .center_x(Length::Fill),

            container(
                toggler(self.params.prismatine_params.invert_phase.value())
                    .on_toggle(Message::SwitchInvPhase)
                    .label("invert phase mode")
                    .width(Length::Fill),
            )
            .width(Length::Fill)
        ]
        .width(Length::Fill)
        .spacing(5.0)
        .into(),

        _ => text("OwO nya").into(),
    }
}

}