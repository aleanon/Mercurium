pub mod choose_recipient;

use std::collections::BTreeMap;

use iced::{
    widget::{self, button, row, text, Button, Container, PickList },
    Alignment, Element, Length, Padding,
};
use ravault_iced_theme::styles::{self, rule::TextInputRule, text_input::TextInput};
use crate::{app::App, message::{app_view_message::transaction_message::TransactionMessage, common_message::CommonMessage, Message}};
use types::{Account, AccountAddress, ResourceAddress};

use self::choose_recipient::ChooseRecipient;


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

#[derive(Debug, Clone)]
pub enum View {
    Transaction,
    ChooseResource(AccountAddress),
    ChooseRecipient(ChooseRecipient),
}

#[derive(Debug, Clone)]
pub struct TransactionView {
    pub(crate) from_account: Option<Account>,
    pub(crate) recipients: Vec<Recipient>,
    pub(crate) message: String,
    pub(crate) view: View,
}

impl TransactionView {
    pub fn new() -> Self {
        Self {
            from_account: None,
            recipients: vec![Recipient::new(None)],
            message: String::new(),
            view: View::Transaction,
        }
    }

    pub fn from_account(account: Account) -> Self {
        Self {
            from_account: Some(account),
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
            View::ChooseRecipient(ref recipient) => recipient.view(app),
            _ => widget::column!().into(),
        }
    }

    fn overview(&'a self, app: &'a App) -> Element<'a, Message> {
        let db = app.db.as_ref().unwrap_or_else(|| {
            unreachable!("{}:{} Database does not exist", module_path!(), line!())
        });

        let accounts_map = db.get_accounts_map().unwrap_or(BTreeMap::new());
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

        let recipient_field = self.recipients();

        let add_recipient = button("Add recipient")
            .width(Length::Fill)
            .height(Length::Shrink)
            .on_press(TransactionMessage::AddRecipient.into()); 

        let space3 = widget::Space::new(Length::Fill, 50);

        let message_field = self.message(app);

        let fields = widget::column![header, space, from_account_field, space2, recipient_field, add_recipient, space3, message_field]
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

    fn recipients(&'a self) -> Container<'a, Message> {
        let label = Self::field_label("TO");
        
            //create empty recipient

        let mut recipients: Vec<Element<'a, Message>> = Vec::with_capacity(self.recipients.len());

        let recipients_iter = self.recipients.iter().enumerate();

        if let Some((i, recipient)) = recipients_iter.next() {
            let recipient_address = recipient.address.and_then(|address| Some(address.truncate_long()))
                .unwrap_or("Choose recipient".to_owned());

            let address = text(recipient_address).size(15).line_height(1.5).width(Length::Shrink);
            let space = widget::Space::new(Length::Fill, 1);
            let remove_recipient = button(widget::Space::new(15, 15)).width(Length::Shrink).height(Length::Shrink)
                .on_press_maybe(recipient.address.and_then(|_| Some(TransactionMessage::RemoveRecipient(i).into())));
            let button_row = row![address, space, remove_recipient].width(Length::Fill).height(Length::Shrink).align_items(Alignment::Center);
            let choose_account = button(botton_row).width(Length::Fill).height(Length::Shrink);

            let 

        }

        for recipient in self.recipients.iter() {
            let recipient_address = recipient.address.and_then(|address| Some(address.truncate_long()))
                .unwrap_or("Choose recipient".to_owned());

            let address = text(recipient.address)
                .size(15)
                .line_height(1.5)
                .width(Length::Fill);

            
            let remove_recipient = widget::button(widget::Space::new(8, 8))
                .width(Length::Shrink)
                .height(Length::Shrink)
                .on_press(TransactionMessage::RemoveRecipient(recipient.address.clone()).into());

            let address_field = row![address, remove_recipient];

            let add_resource = widget::button(widget::Space::new(5,5))
                .height(10)
                .width(10);

            let mut assets:Vec<Element<'a, Message>> = Vec::with_capacity(recipient.resources.len());

            for (symbol, resource_address, amount) in recipient.resources.iter() {
                //placeholder for actual icon
                let icon = widget::Space::new(10, 10);

                let symbol = Self::resource_text_field(&symbol);

                let address = Self::resource_text_field(&resource_address.truncate());

                let amount = widget::text_input("Amount", &amount)
                    .on_input(|input| TransactionMessage::UpdateResourceAmount((recipient.address.clone(), resource_address.clone(), input)).into())
                    .on_paste(|input| TransactionMessage::UpdateResourceAmount((recipient.address.clone(), resource_address.clone(), input)).into());

                let remove_resource = widget::button(widget::Space::new(5, 5));

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

            let recipient = widget::column![address_field, assets, add_resource]
                .spacing(2)
                .width(Length::Fill)
                .height(Length::Shrink);

            recipients.push(recipient.into())
        }

            let recipients = widget::column(recipients);

            return widget::container(widget::column![label, recipients])
        

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
