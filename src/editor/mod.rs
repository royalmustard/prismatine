
use std::sync::Arc;

use crate::editor::parameter_box::create_parameter_box;
use crate::util::ProcessMode;
use crate::PrismatineParams;
use atomic_refcell::AtomicRefCell;
use nih_plug::nih_dbg;
use nih_plug::params::Param;
use nih_plug::prelude::AtomicF32;
use nih_plug::{editor::Editor, prelude::GuiContext};
use nih_plug_iced::core::Element;
use nih_plug_iced::widget::{canvas, container, pick_list, toggler, Column, Row, Text};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use nih_plug_iced::{create_iced_editor, IcedEditor, IcedState};
use seven_segment_iced::canvas_segment::SevenSegmentCanvas;
use seven_segment_iced::SevenSegmentStyle;

mod parameter_box;

pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(400, 500)
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
    FontLoaded,
    ParamUpdate(nih_widgets::ParamMessage),
    SwitchInvPhase(bool),
    ProcessModeSelected(ProcessMode)
}

struct PrismatineEditor {
    params: PrismatineEditorParams,
    context: Arc<dyn GuiContext>,

    I_c_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
    phase_gain_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,

    process_mode_state: Option<ProcessMode>,
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
            process_mode_state: Some(ProcessMode::Josephson),
        };

        (editor, font::load(include_bytes!("/usr/share/fonts/TTF/Comic.TTF").as_slice()).map(|_| Message::FontLoaded))
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
            },
            Message::ProcessModeSelected(pm) => {self.process_mode_state = Some(pm)},
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Theme, Renderer> {

        let process_modes = [ProcessMode::Josephson, ProcessMode::AB, ProcessMode::KO1, ProcessMode::KO2];


        let left_column = Column::new().spacing(5.0)
            .push(Text::new("Prismatine")
                        .size(30.0)
                        .font(Font::with_name("Noto Sans"))
                        .center()
                        .width(Length::Fill))
            .push(container(
                pick_list(process_modes, self.process_mode_state, Message::ProcessModeSelected)
                .font(Font::with_name("Noto Sans"))
                .placeholder("nyaaaa")
                )
                .width(Length::Fill)
                .center_x(Length::Fill))
            .push(Text::new("phase gain").width(Length::Fill).center())
            .push(container(nih_plug_iced::widgets::ParamSlider::new(self.phase_gain_slider_state.clone(), &self.params.prismatine_params.phase_gain).map(Message::ParamUpdate)).width(Length::Fill).center_x(Length::Fill))
            .push(Text::new("critical current").width(Length::Fill).center())
            .push(container(nih_plug_iced::widgets::ParamSlider::new(self.I_c_slider_state.clone(), &self.params.prismatine_params.I_c).map(Message::ParamUpdate)).width(Length::Fill).center_x(Length::Fill))
            .push(container(
                toggler(self.params.prismatine_params.invert_phase.value())
                .on_toggle(Message::SwitchInvPhase)
                .label("invert phase mode")
                .width(Length::Fill)
            )
            .width(Length::Fill));
            //.push(Text::new(format!("{:?}", )));

            let parameter_box = create_parameter_box();

            Row::new()
            .push(left_column)
            .push(parameter_box)
            .height(Length::Fill)
            .into()
    }
}
