use std::sync::Arc;

use nih_plug::{editor::Editor, prelude::GuiContext};
use nih_plug_iced::{create_iced_editor, IcedEditor, IcedState};
use nih_plug_iced::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::widget::{Column, Space, Text};
use crate::PrismatineParams;


pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(200, 150)
}

pub(crate) fn create(
    params: Arc<PrismatineParams>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>>
{
    create_iced_editor::<PrismatineEditor>(editor_state, (params))
}


#[derive(Debug, Clone, Copy)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),
}


struct PrismatineEditor
{
    params: Arc<PrismatineParams>,
    context: Arc<dyn GuiContext>,
}

impl IcedEditor for PrismatineEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<PrismatineParams>);
    
    fn new(
        (params): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Task<Self::Message>) {
        let editor = PrismatineEditor
        {
            params,
            context,
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
        }

        Task::none()
    }
    
    fn view(&self) -> Element<'_, Self::Message> {
        Column::new().push(Text::new("Gadse nya meow meeeerrrrp :3")).into()
    }
    
   
}