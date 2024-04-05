pub mod choose_recipient;
pub mod add_assets;

use std::collections::BTreeMap;

use iced::{
    advanced::graphics::core::window::icon, widget::{self, button, image::Handle, row, text, Button, Container, PickList }, Alignment, Element, Length, Padding
};
use ravault_iced_theme::styles::{self, rule::TextInputRule, text_input::TextInput};
use crate::{app::App, message::{app_view_message::transaction_message::TransactionMessage, common_message::CommonMessage, Message}};
use types::{Account, AccountAddress, ResourceAddress};

use self::{add_assets::AddAssets, choose_recipient::ChooseRecipient};

use super::accounts_view::account_view::fungible_view::NO_IMAGE_ICON;


// pub struct TransactionView {
//     from_account: Option<String>,
//     recipient: Recipient,
//     message: Option<String>,
// }

const PICK_LIST_DEFAULT_TEXT: &'static str = "Select account";

#[derive(Debug, Clone)]
pub struct Recipient {
    pub(crate) address: Option<AccountAddress>,
    pub(crate) resources: Vec<(String, ResourceAddress, String)>,
}

impl Recipient {
    pub fn new(address: Option<AccountAddress>) -> Self {
        Self {
            address,
            resources: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum View {
    Transaction,
    ChooseResource(AddAssets),
    ChooseRecipient(ChooseRecipient),
}

#[derive(Debug)]
pub struct TransactionView {
    pub(crate) from_account: Option<Account>,
    pub(crate) recipients: Vec<Recipient>,
    pub(crate) message: String,
    pub(crate) view: View,
}

impl TransactionView {
    pub fn new(from_account: Option<Account>) -> Self {
        Self {
            from_account,
            recipients: vec![Recipient::new(None)],
            message: String::new(),
            view: View::Transaction,
        }
    }

    pub fn from_recipient(address: AccountAddress) -> Self {
        Self {
            from_account: None,
            recipients: vec![Recipient::new(Some(address))],
            message: String::new(),
            view: View::Transaction,
        }
    }
}

impl<'a> TransactionView {
    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        match self.view {
            View::Transaction => self.overview(app),
            View::ChooseRecipient(ref choose_recipient) => choose_recipient.view(app),
            _ => widget::column!().into(),
        }
    }

    fn overview(&'a self, app: &'a App) -> Element<'a, Message> {

        let accounts_map = app.app_data.db.get_accounts_map().unwrap_or(BTreeMap::new());
        let mut accounts = accounts_map
            .values()
            .map(|account| account.clone())
            .collect::<Vec<Account>>();
        // The accounts are sorted by ID
        accounts.sort_unstable_by(|a, b| a.cmp(b));

        let header = Self::header("Transaction");
        let space = widget::Space::new(Length::Fill, 50);
        
        let from_account_field = self.from_account_field(accounts);
        let space2 = widget::Space::new(Length::Fill, 30);

        let recipient_field = self.recipients(app);

        let space3 = widget::Space::new(Length::Fill, 20);

        let add_recipient = button("Add recipient")
            .width(Length::Fill)
            .height(Length::Shrink)
            .on_press(TransactionMessage::AddRecipient.into()); 

        let space4 = widget::Space::new(Length::Fill, 50);

        let message_field = self.message(app);

        let fields = widget::column![header, space, from_account_field, space2, recipient_field, space3, add_recipient, space4, message_field]
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(Padding {
                left: 10.,
                right: 10.,
                ..Padding::ZERO
            });

        let scrollable = widget::scrollable(fields).width(400);

        // let left_space = widget::Space::new(Length::Fill, Length::Fill);
        // let right_space = widget::Space::new(Length::Fill, Length::Fill);

        // let content = widget::row![left_space, scrollable, right_space];
        widget::container(scrollable)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .padding(Padding {
                left: 15.,
                right: 15.,
                ..Padding::ZERO
            })
            .into()
    }

    fn message(&self, app: &'a App) -> Container<'a, Message> {
        let label = Self::field_label("Message");

        let text_field = widget::text_input("Enter Message", &self.message)
            .size(12)
            .line_height(1.5)
            .on_input(|message| TransactionMessage::UpdateMessage(message).into())
            .on_paste(|message| TransactionMessage::UpdateMessage(message).into())
            .style(iced::theme::TextInput::Custom(Box::new(styles::text_input::TextInput)));

        let rule = widget::Rule::horizontal(4)
            .style(TextInputRule::style);

        let col = widget::column![label, text_field, rule].align_items(Alignment::Start);
    
        widget::container(col)
    }

    fn from_account_field(
        &'a self,
        accounts: Vec<Account>,
    ) -> Container<'a, Message> {
        let label = Self::field_label("From");

        let (account_name, account_address) = match self.from_account {
            Some(ref account) => (account.name.as_str(), account.address.truncate()),
            None => ("Select Account", String::with_capacity(0)),
        };

        let picklist = widget::pick_list(
            accounts, 
            self.from_account.as_ref(),
            |account| TransactionMessage::SelectAccount(account).into()
        )
        .placeholder("Select account")
        .text_line_height(2.)
        .text_size(12)
        .width(Length::Fill);

        let col = widget::column![label, picklist]
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::Start);

        widget::container(col)
            .width(Length::Fill)
            .height(Length::Shrink)
    }

    fn recipients(&'a self, app: &'a App) -> Container<'a, Message> {
        let label = Self::field_label("TO");
        
            //create empty recipient

        let mut recipients: Vec<Element<'a, Message>> = Vec::with_capacity(self.recipients.len());

        for (recipient_index, ref recipient) in self.recipients.iter().enumerate() {
            let recipient_address = recipient.address.as_ref().and_then(|address| Some(address.truncate_long()))
                .unwrap_or("Choose recipient".to_owned());

            let address = text(recipient_address)
                .size(15)
                .line_height(1.5)
                .width(Length::Fill);

            
            let mut remove_recipient = widget::button(widget::Space::new(8, 8))
                .width(Length::Shrink)
                .height(Length::Shrink);

            if recipient_index != 0 {
                remove_recipient = remove_recipient.on_press(TransactionMessage::RemoveRecipient(recipient_index).into());
            } else {
                remove_recipient = remove_recipient.on_press_maybe(recipient.address.as_ref().and_then(|_| Some(TransactionMessage::RemoveRecipient(recipient_index).into())));
            }

            let choose_recipient_content = row![address, remove_recipient];

            let choose_recipient = button(choose_recipient_content).width(Length::Fill).height(Length::Shrink)
                .on_press(TransactionMessage::SelectRecipient(recipient_index).into());

            let add_resource = widget::button(
                text("Add assets")
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center)
            )
            .height(20)
            .width(Length::Fill)
            ;

            let mut assets:Vec<Element<'a, Message>> = Vec::with_capacity(recipient.resources.len());

            for (resource_index, (symbol, resource_address, amount)) in recipient.resources.iter().enumerate() {
                //placeholder for actual icon
                let icon_handle = app.appview.resource_icons.get(&resource_address)
                    .and_then(|handle| Some(handle.clone()))
                    .unwrap_or(Handle::from_memory(NO_IMAGE_ICON));

                let icon = widget::image(icon_handle).width(20).height(20);

                let symbol = Self::resource_text_field(&symbol);

                let address = Self::resource_text_field(&resource_address.truncate());

                let amount = widget::text_input("Amount", &amount)
                    .on_input(move |input| TransactionMessage::UpdateResourceAmount(recipient_index, resource_index, input).into())
                    .on_paste(move |input| TransactionMessage::UpdateResourceAmount(recipient_index, resource_index, input).into());

                let remove_resource = widget::button(
                    text("x")
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .size(10)
                    )
                    .height(10)
                    .width(10);

                let resource_row = row![icon, symbol, address, amount, remove_resource]
                    .spacing(2)
                    .height(Length::Shrink)
                    .width(Length::Fill);
                
                let resource = widget::column![
                    widget::Rule::horizontal(1),
                    resource_row
                ]
                .height(Length::Shrink)
                .width(Length::Fill);

                assets.push(resource.into())
            }

            let assets= widget::column(assets)
                .spacing(1)
                .width(Length::Fill);

            let recipient = widget::column![choose_recipient, assets, add_resource]
                .spacing(2)
                .width(Length::Fill)
                .height(Length::Shrink);

            recipients.push(recipient.into())
        }

        let recipients = widget::column(recipients);

        widget::container(widget::column![label, recipients])
    }

    fn resource_text_field(str: &str) -> widget::Text<'a> {
        text(str)
            .size(10)
            .line_height(1.)
            .width(Length::Shrink)
    }

    fn header(input: &str) -> text::Text<'a> {
        text(input)
            .line_height(2.)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .size(20)
    }

    fn field_label(input: &str) -> text::Text<'a> {
        text(input)
            .line_height(2.)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .width(Length::Fill)
    }

}
