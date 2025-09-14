use nih_plug_iced::{widget::{container, text}, Element, Length, Renderer, Theme};

use crate::editor::Message;

pub fn create_parameter_box() ->  Element<'static, Message, Theme, Renderer> 
{
    container(text("OwO amogus"))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}