use std::f32;
use std::sync::Arc;

use crate::PrismatineParams;
use atomic_refcell::AtomicRefCell;
use nih_plug::nih_dbg;
use nih_plug::params::Param;
use nih_plug::prelude::AtomicF32;
use nih_plug::{editor::Editor, prelude::GuiContext};
use nih_plug_iced::core::Element;
use nih_plug_iced::widget::{canvas, container, toggler, Column, Text};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use nih_plug_iced::{create_iced_editor, IcedEditor, IcedState};
use seven_segment_iced::canvas_segment::SevenSegmentCanvas;
use seven_segment_iced::SevenSegmentStyle;

pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(200, 500)
}

pub(crate) fn create(
    params: PrismatineEditorParams,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<PrismatineEditor>(editor_state, params)
}

#[derive(Debug, Clone, Copy)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),
    SwitchInvPhase(bool)
}

struct PrismatineEditor {
    params: PrismatineEditorParams,
    context: Arc<dyn GuiContext>,

    I_c_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
    phase_gain_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
}

#[derive(Clone)]
pub struct PrismatineEditorParams {
    pub prismatine_params: Arc<PrismatineParams>,
    pub phase: Arc<[AtomicF32; 2]>,
}

impl IcedEditor for PrismatineEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = PrismatineEditorParams;

    fn new(
        params: Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Task<Self::Message>) {
        let editor = PrismatineEditor {
            params,
            context,
            I_c_slider_state: Default::default(),
            phase_gain_slider_state: Default::default(),
        };

        (editor, Task::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        //window: &mut WindowQueue,
        message: Self::Message,
    ) -> Task<Self::Message> {
        match message {
            Message::ParamUpdate(message) => self.handle_param_message(message),
            Message::SwitchInvPhase(value) => {
                unsafe {
                    self.context.raw_begin_set_parameter(self.params.prismatine_params.invert_phase.as_ptr());
                    self.context.raw_set_parameter_normalized(self.params.prismatine_params.invert_phase.as_ptr(), match value {true => 1.0, _ => 0.0});
                    self.context.raw_end_set_parameter(self.params.prismatine_params.invert_phase.as_ptr());
                }
                
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Theme, Renderer> {
        let phase_left = self.params.phase[0]
            .load(std::sync::atomic::Ordering::Relaxed)
            .rem_euclid(f32::consts::PI)
            .to_degrees();
        let phase_right = self.params.phase[1]
            .load(std::sync::atomic::Ordering::Relaxed)
            .rem_euclid(f32::consts::PI)
            .to_degrees();

        Column::new().spacing(5.0)
            .push(Text::new("Prismatine")
                        .size(30.0)
                        .font(Font::with_name("NotoSans"))
                        .center()
                        .width(Length::Fill))
            .push(
                canvas(SevenSegmentCanvas::new(
                    seven_segment_iced::glyph::string_with_decimals_to_segment(format!(
                        "{phase_left:0>5.1}"
                    )),
                    4,
                    SevenSegmentStyle {
                        background_color: Color::from_rgb(0.047, 0.067, 0.09),
                        segment_color: Color::from_rgb(0.69, 1.0, 0.996),
                        off_color: None, //Color or inactive segments
                        margin_frac: 1.0 / 15.0,
                        aspect_ratio: 6.9,
                        line_margin_frac: 1.0 / 30.0,
                        dot_size_frac: 1.0 / 15.0,
                    },
                ))
                .width(Length::Fill),
            )
            .push(
                canvas(SevenSegmentCanvas::new(
                    seven_segment_iced::glyph::string_with_decimals_to_segment(format!(
                        "{phase_right:0>5.1}"
                    )),
                    4,
                    SevenSegmentStyle {
                        background_color: Color::from_rgb(0.047, 0.067, 0.09),
                        segment_color: Color::from_rgb(0.69, 1.0, 0.996),
                        off_color: None, //Color or inactive segments
                        margin_frac: 1.0 / 15.0,
                        aspect_ratio: 6.9,
                        line_margin_frac: 1.0 / 30.0,
                        dot_size_frac: 1.0 / 15.0,
                    },
                ))
                .width(Length::Fill),
            )
            .push(Text::new("phase gain").width(Length::Fill).center())
            .push(container(nih_plug_iced::widgets::ParamSlider::new(self.phase_gain_slider_state.clone(), &self.params.prismatine_params.phase_gain).map(Message::ParamUpdate)).width(Length::Fill).center_x(Length::Fill))
            .push(Text::new("critical current").width(Length::Fill).center())
            .push(container(nih_plug_iced::widgets::ParamSlider::new(self.I_c_slider_state.clone(), &self.params.prismatine_params.I_c).map(Message::ParamUpdate)).width(Length::Fill).center_x(Length::Fill))
            .push(container(
                toggler(self.params.prismatine_params.invert_phase.value())
                .on_toggle(Message::SwitchInvPhase)
                .label("Invert phase mode")
                .width(Length::Fill)
            ).width(Length::Fill))
            .into()
    }
}
