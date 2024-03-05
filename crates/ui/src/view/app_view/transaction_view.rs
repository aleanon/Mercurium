use std::collections::BTreeMap;

use iced::{
    widget::{self, text, row, Button, Container },
    Alignment, Element, Length, Padding,
};

use crate::{app::App, message::{app_view_message::transaction_message::TransferMessage, common_message::CommonMessage, Message}};
use types::{Account, AccountAddress, ResourceAddress};


// pub struct TransactionView {
//     from_account: Option<String>,
//     recipient: Recipient,
//     message: Option<String>,
// }

const PICK_LIST_DEFAULT_TEXT: &'static str = "Select account";

#[derive(Debug, Clone)]
pub struct Recipient {
    pub(crate) recipient_chooser: bool,
    pub(crate) address: AccountAddress,
    pub(crate) recource_chooser: bool,
    pub(crate) resources: Vec<(String, ResourceAddress, String)>,
}

impl Recipient {
    pub fn new() -> Self {
        Self {
            recipient_chooser: false,
            address: AccountAddress::empty(),
            recource_chooser: false,
            resources: Vec::new(),
        }
    }

    pub fn from_account(account: AccountAddress) -> Self {
        Self {
            recipient_chooser: false,
            address: account,
            recource_chooser: false,
            resources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum View {
    OverView,
    ChooseFromAccount,
    ChooseResource(AccountAddress),
    Message(String),
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
            recipients: Vec::new(),
            message: String::new(),
            view: View::OverView,
        }
    }

    pub fn from_account(account: Account) -> Self {
        Self {
            from_account: Some(account),
            recipients: Vec::new(),
            message: String::new(),
            view: View::OverView,
        }
    }

    pub fn from_recipient(recipient: AccountAddress) -> Self {
        Self {
            from_account: None,
            recipients: vec![Recipient::from_account(recipient)],
            message: String::new(),
            view: View::OverView,
        }
    }
}

impl<'a> TransactionView {
    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        match self.view {
            View::OverView => self.overview(app),
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
        // The account are sorted by ID
        accounts.sort_unstable_by(|a, b| a.cmp(b));

        let header = Self::top_label("Transaction");
        let space = widget::Space::new(Length::Fill, 50);
        let from_account_field = self.from_account_field(accounts, accounts_map);
        let recipient_field = self.recipient_field();

        let space2 = widget::Space::new(Length::Fill, 40);

        let message_field = self.message(app);

        let fields = widget::column![header, space, from_account_field, recipient_field, space2, message_field]
            .width(Length::Fill)
            .height(Length::Shrink)
            .padding(Padding {
                left: 10.,
                right: 10.,
                ..Padding::ZERO
            });

        let scrollable = widget::scrollable(fields).width(400);

        let left_space = widget::Space::new(Length::Fill, Length::Fill);
        let right_space = widget::Space::new(Length::Fill, Length::Fill);

        let content = widget::row![left_space, scrollable, right_space];
        widget::container(content)
            .width(300)
            .height(Length::Fill)
            .width(Length::Fill)
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
            .on_input(|message| TransferMessage::UpdateMessage(message).into())
            .on_paste(|message| TransferMessage::UpdateMessage(message).into());

        let col = widget::column![label, text_field].align_items(Alignment::Start);

        widget::container(col)
    }

    fn from_account_field(
        &self,
        accounts: Vec<Account>,
        accounts_map: BTreeMap<AccountAddress, Account>,
    ) -> Container<'a, Message> {
        let label = Self::field_label("From");

        let (account_name, account_address) = match self.from_account {
            Some(ref account) => (account.name.as_str(), account.address.truncate()),
            None => ("Select Account", String::with_capacity(0)),
        };

        let account_name = text(account_name)
            .size(15)
            .line_height(1.5)
            .width(Length::Shrink)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let account_address = text(account_address)
            .size(12)
            .line_height(1.5)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .vertical_alignment(iced::alignment::Vertical::Bottom);

        let account_address_icon = widget::Space::new(10, 5);

        let account_address_row = widget::row![account_address, account_address_icon].spacing(5);

        let account_address_button = Button::new(account_address_row)
            .on_press_maybe(
            self.from_account.as_ref().and_then(|account| {
                Some(Message::Common(CommonMessage::CopyToClipBoard(
                    account.address.to_string(),
                )))
            }),
        );

        let space = widget::Space::new(Length::Fill, 1);

        let account_name_address = widget::row![account_name, space, account_address_button]
            .spacing(15)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_items(Alignment::End);

        let rule = widget::Rule::horizontal(1);

        let name_address_button = widget::button(account_name_address).on_press(TransferMessage::ChooseAccount.into())
            .width(Length::Fill);

        let col = widget::column![label, rule, name_address_button]
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_items(Alignment::Start);

        widget::container(col)
            .width(Length::Fill)
            .height(Length::Shrink)
    }

    fn recipient_field(&'a self) -> Container<'a, Message> {
        let label = Self::field_label("Recipient");
        
        if self.recipients.len() == 0 {
            //create empty recipient
            let choose_recipient = widget::button(
                widget::text("Choose account")
                    .size(12)
                    .line_height(1.5)
                );
                let select_asset = widget::button(
                    widget::text("Select asset")
                    .size(12)
                    .line_height(1.5)   
                ); 
            //TODO: On press open modal to choose account
            widget::container(widget::column![label, choose_recipient, select_asset])
            .width(Length::Fill)    
        } else {

            let mut recipients: Vec<Element<'a, Message>> = Vec::with_capacity(self.recipients.len());

            for recipient in self.recipients.iter() {
                let address = text(recipient.address.truncate())
                    .size(15)
                    .line_height(1.5)
                    .width(Length::Fill);

                let remove_recipient = widget::button(widget::Space::new(8, 8))
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .on_press(TransferMessage::RemoveRecipient(recipient.address.clone()).into());

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
                        .on_input(|input| TransferMessage::UpdateResourceAmount((recipient.address.clone(), resource_address.clone(), input)).into())
                        .on_paste(|input| TransferMessage::UpdateResourceAmount((recipient.address.clone(), resource_address.clone(), input)).into());

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

    }

    fn resource_text_field(str: &str) -> widget::Text<'a> {
        text(str)
            .size(10)
            .line_height(1.)
            .width(Length::Shrink)
    }

    fn top_label(input: &str) -> text::Text<'a> {
        Self::label(input).size(15)
    }

    fn field_label(input: &str) -> text::Text<'a> {
        Self::label(input).size(12)
    }

    fn label(input: &str) -> text::Text<'a> {
        text(input)
            .line_height(2.)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .width(Length::Fill)
    }
}
