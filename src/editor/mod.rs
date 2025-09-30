use std::sync::Arc;


use crate::util::ProcessMode;
use crate::PrismatineParams;
use atomic_refcell::AtomicRefCell;
use nih_plug::nih_dbg;
use nih_plug::params::Param;
use nih_plug::prelude::{AtomicF32, ParamSetter};
use nih_plug::{editor::Editor, prelude::GuiContext};
use nih_plug_iced::core::Element;
use nih_plug_iced::widget::{canvas, column, container, pick_list, row, text, toggler, Column, Row, Text};
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
    ProcessModeSelected(ProcessMode),
}

struct PrismatineEditor {
    params: PrismatineEditorParams,
    context: Arc<dyn GuiContext>,

    I_c_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
    phase_gain_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
    temperature_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
    terms_slider_state: Arc<AtomicRefCell<nih_widgets::param_slider::State>>,
}

#[derive(Clone)]
pub struct PrismatineEditorParams {
    pub prismatine_params: Arc<PrismatineParams>,
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
            temperature_slider_state: Default::default(),
            terms_slider_state: Default::default(),
        };

        (
            editor,
            Task::none(),
        )
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
            Message::SwitchInvPhase(value) =>  {
                let setter = ParamSetter::new(&*self.context);
                setter.begin_set_parameter(&self.params.prismatine_params.invert_phase);
                setter.set_parameter(&self.params.prismatine_params.invert_phase, value);
                setter.end_set_parameter(&self.params.prismatine_params.invert_phase);
            },
            Message::ProcessModeSelected(pm) =>  {
                let setter = ParamSetter::new(&*self.context);
                setter.begin_set_parameter(&self.params.prismatine_params.process_mode);
                setter.set_parameter(&self.params.prismatine_params.process_mode, pm);
                setter.end_set_parameter(&self.params.prismatine_params.process_mode);
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Theme, Renderer> {
        let process_modes = [
            ProcessMode::Josephson,
            ProcessMode::AB,
            ProcessMode::KO1,
            ProcessMode::KO2,
        ];

        let left_column = column![
            Text::new("Prismatine")
                .size(30.0)
                .font(Font::with_name("Noto Sans"))
                .center()
                .width(Length::Fill),

            pick_list(
                process_modes,
                Some(self.params.prismatine_params.process_mode.value()),
                Message::ProcessModeSelected
            )
            .width(Length::Fill),

        ]
        .spacing(5.0)
        .width(Length::FillPortion(1));



        let parameter_box = self.create_parameter_box();

        row![left_column, parameter_box]
            .height(Length::Fill)
            .into()
    }
}
