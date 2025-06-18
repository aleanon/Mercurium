use deps::{
    iced::{
        alignment::Horizontal,
        widget::{column, Button, Column, Rule, Space},
    },
    *,
};

use std::collections::HashMap;

use crate::{app::AppMessage, components, unlocked::app_view};
use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, button, container, image::Handle, row, text, Container},
    Alignment, Element, Length, Padding, Task,
};
use types::{
    address::{AccountAddress, Address, ResourceAddress},
    Account, Decimal,
};
use wallet::{Unlocked, Wallet};

use super::{
    add_assets::{self, AddAssets},
    add_recipient::{self, AddRecipient},
};

#[derive(Debug, Clone)]
pub enum Message {
    OverView,
    SelectAccount(Account),
    // UpdateTextMessage(Edit),
    RemoveRecipient(usize),
    UpdateResourceAmount(usize, ResourceAddress, String),
    SelectRecipient(usize),
    AddRecipient,
    ChooseRecipientMessage(add_recipient::Message),
    ///Pass the index of the account to add assets for
    AddAssets(usize),
    AddAssetsMessage(add_assets::Message),
    TextFieldMessage(components::text_field::Message),
    RemoveAsset(usize, ResourceAddress),
}

impl Into<AppMessage> for Message {
    fn into(self) -> AppMessage {
        AppMessage::AppView(app_view::Message::TransactionMessage(self))
    }
}

#[derive(Debug, Clone)]
pub struct Recipient {
    pub(crate) address: Option<AccountAddress>,
    pub(crate) resources: HashMap<ResourceAddress, (String, String)>,
}

impl Recipient {
    pub fn new(address: Option<AccountAddress>) -> Self {
        Self {
            address,
            resources: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum View {
    Transaction,
    AddAssets(AddAssets),
    ChooseRecipient(AddRecipient),
}

#[derive(Debug)]
pub struct CreateTransaction {
    pub(crate) from_account: Option<Account>,
    pub(crate) resource_amounts: HashMap<ResourceAddress, Decimal>,
    pub(crate) recipients: Vec<Recipient>,
    pub(crate) text_field: components::text_field::TextField,
    pub(crate) view: View,
}

impl CreateTransaction {
    pub fn new(
        from_account: Option<Account>,
        account_resources: Option<HashMap<ResourceAddress, Decimal>>,
    ) -> Self {
        Self {
            from_account,
            resource_amounts: account_resources.unwrap_or(HashMap::new()),
            recipients: vec![Recipient::new(None)],
            text_field: components::text_field::TextField::new(),
            view: View::Transaction,
        }
    }

    pub fn from_recipient(address: AccountAddress) -> Self {
        Self {
            from_account: None,
            resource_amounts: HashMap::new(),
            recipients: vec![Recipient::new(Some(address))],
            text_field: components::text_field::TextField::new(),
            view: View::Transaction,
        }
    }
}

impl<'a> CreateTransaction {
    pub fn update(
        &mut self,
        message: Message,
        wallet: &'a mut Wallet<Unlocked>,
    ) -> Task<AppMessage> {
        match message {
            Message::OverView => self.view = View::Transaction,
            Message::SelectAccount(account) => self.from_account = Some(account),
            Message::RemoveRecipient(recipient_index) => self.remove_recipient(recipient_index),
            Message::UpdateResourceAmount(account_index, resource, amount) => {
                self.update_resource_amount(account_index, resource, amount)
            }
            Message::SelectRecipient(recipient_index) => {
                let from_address = self
                    .from_account
                    .as_ref()
                    .and_then(|account| Some(account.address.clone()));
                self.view = View::ChooseRecipient(AddRecipient::new(recipient_index, from_address))
            }
            Message::AddRecipient => self.recipients.push(Recipient::new(None)),
            Message::AddAssets(recipient_index) => {
                if let Some(account) = &self.from_account {
                    self.create_new_add_assets_view(recipient_index, account.address.clone())
                }
            }
            Message::AddAssetsMessage(add_assets_message) => {
                if let View::AddAssets(add_assets) = &mut self.view {
                    return add_assets.update(add_assets_message, &mut self.recipients, wallet);
                }
            }
            Message::ChooseRecipientMessage(choose_recipient_message) => {
                if let View::ChooseRecipient(choose_recipient) = &mut self.view {
                    return choose_recipient.update(
                        choose_recipient_message,
                        &mut self.recipients,
                        wallet,
                    );
                }
            }
            Message::RemoveAsset(recipient_index, resource_address) => {
                self.recipients[recipient_index]
                    .resources
                    .remove(&resource_address);
            }
            Message::TextFieldMessage(message) => self.text_field.update(message),
        }

        Task::none()
    }

    fn create_new_add_assets_view(&mut self, recipient_index: usize, from_account: AccountAddress) {
        let selected = self.recipients[recipient_index].resources.clone();
        self.view = View::AddAssets(AddAssets::new(from_account, recipient_index, selected))
    }

    fn remove_recipient(&mut self, index: usize) {
        if self.recipients.len() == 1 {
            self.recipients[index].address = None;
        } else if self.recipients.len() > index {
            self.recipients.remove(index);
        }
    }

    fn update_resource_amount(
        &mut self,
        account_index: usize,
        resource_address: ResourceAddress,
        new_amount: String,
    ) {
        if new_amount.parse::<f32>().is_ok() || new_amount.is_empty() {
            if let Some((_, amount)) = self.recipients[account_index]
                .resources
                .get_mut(&resource_address)
            {
                *amount = new_amount;
            }
        }
    }

    pub fn view(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        match &self.view {
            View::Transaction => self.create_transaction(wallet),
            View::ChooseRecipient(choose_recipient) => choose_recipient.view(wallet),
            View::AddAssets(choose_assets) => choose_assets.view(wallet),
        }
    }

    fn create_transaction(&'a self, wallet: &'a Wallet<Unlocked>) -> Element<'a, AppMessage> {
        let mut accounts = wallet
            .accounts()
            .values()
            .map(|account| account)
            .collect::<Vec<&Account>>();

        // The accounts are sorted by ID
        accounts.sort_unstable_by(|a, b| a.cmp(b));

        let header = Self::header("Transaction");

        let from_account_field = self.from_account_field(accounts);

        let recipient_field = self.recipients(wallet);

        let add_recipient = row![
            Space::new(Length::FillPortion(2), 1),
            button(text("Add recipient").center())
                .padding(5)
                .width(Length::FillPortion(6))
                .height(Length::Shrink)
                .style(styles::button::choose_account)
                .on_press(Message::AddRecipient.into()),
            Space::new(Length::FillPortion(2), 1)
        ];

        let message_field = self.message();

        let fields = widget::column![
            header,
            from_account_field,
            recipient_field,
            add_recipient,
            message_field
        ]
        .spacing(30)
        .height(Length::Shrink)
        .padding(Padding {
            left: 10.,
            right: 15.,
            ..Padding::ZERO
        });

        let scrollable = widget::scrollable(fields).style(styles::scrollable::vertical_scrollable);

        let page_top = widget::container(scrollable)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding {
                left: 5.,
                right: 5.,
                bottom: 10.,
                ..Padding::ZERO
            });

        let create_transaction = container(
            button(text("Create transaction").width(Length::Fill).center())
                .width(Length::Fill)
                .height(50),
        )
        .padding(Padding {
            left: 15.,
            right: 20.,
            ..Padding::ZERO
        });

        column![page_top, create_transaction].into()
    }

    fn message(&'a self) -> Container<'a, AppMessage> {
        let label = Self::field_label("Message");

        let text_field = self
            .text_field
            .view(|m| Message::TextFieldMessage(m).into())
            .placeholder("Enter Message")
            .padding(10)
            .height(120);

        let col = widget::column![label, text_field]
            .spacing(5)
            .align_x(Alignment::Start);

        widget::container(col)
    }

    fn from_account_field(&'a self, accounts: Vec<&'a Account>) -> Container<'a, AppMessage> {
        let label = Self::field_label("From");

        let (_account_name, _account_address) = match self.from_account {
            Some(ref account) => (account.name.as_str(), account.address.truncate()),
            None => ("Select Account", String::with_capacity(0)),
        };

        let picklist = widget::pick_list(accounts, self.from_account.as_ref(), |account| {
            Message::SelectAccount(account.clone()).into()
        })
        .placeholder("Select account")
        .text_line_height(2.)
        .text_size(14)
        .width(Length::Fill)
        .style(styles::pick_list::from_account)
        .padding(10);

        let col = widget::column![label, picklist]
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Alignment::Start)
            .spacing(5);

        widget::container(col)
            .width(Length::Fill)
            .height(Length::Shrink)
    }

    fn recipients(&'a self, wallet: &'a Wallet<Unlocked>) -> Container<'a, AppMessage> {
        let label = Self::field_label("TO");

        let recipients = self
            .recipients
            .iter()
            .enumerate()
            .map(|(recipient_index, recipient)| self.recipient(recipient_index, recipient, wallet));

        let recipients = column(recipients).spacing(10);

        container(column![label, recipients].spacing(5))
    }

    fn resource_text_field(str: &'a str) -> widget::Text<'a> {
        text(str).size(12).width(Length::Shrink)
    }

    fn header(input: &'a str) -> text::Text<'a> {
        text(input)
            .align_x(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .size(20)
    }

    fn field_label(input: &'a str) -> text::Text<'a> {
        text(input)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Center)
            .width(Length::Fill)
    }

    fn recipient(
        &self,
        recipient_index: usize,
        recipient: &'a Recipient,
        wallet: &'a Wallet<Unlocked>,
    ) -> Element<'a, AppMessage> {
        let choose_recipient_button = Self::choose_recipient(recipient_index, recipient);

        let assets = Self::selected_assets(recipient_index, recipient, wallet);

        let add_assets = widget::button(
            text("Add assets")
                .size(14)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .padding(Padding {
            bottom: 5.,
            top: 5.,
            left: 10.,
            right: 10.,
        })
        .style(styles::button::choose_account)
        .on_press_maybe(
            self.from_account
                .as_ref()
                .and_then(|_| Some(Message::AddAssets(recipient_index).into())),
        );

        let recipient = widget::column![choose_recipient_button, assets, add_assets]
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Horizontal::Center)
            .spacing(10);

        container(recipient)
            .padding(10)
            .style(styles::container::recipient)
            .into()
    }

    fn choose_recipient(
        recipient_index: usize,
        recipient: &'a Recipient,
    ) -> Button<'a, AppMessage> {
        let address = recipient
            .address
            .as_ref()
            .and_then(|address| Some(address.truncate_long()))
            .unwrap_or("Choose account".to_owned());

        let address = text(address).size(15).line_height(1.5).width(Length::Fill);

        let remove_recipient_button = if recipient_index == 0 && recipient.address.is_none() {
            button(text("")).style(button::text)
        } else {
            button(text(Bootstrap::XLg).font(BOOTSTRAP_FONT).line_height(1.))
                .padding(0)
                .style(styles::button::choose_account)
                .on_press(Message::RemoveRecipient(recipient_index).into())
        };

        let choose_recipient_content = row![address, remove_recipient_button]
            .align_y(Alignment::Center)
            .height(Length::Fill);

        button(choose_recipient_content)
            .height(50)
            .padding(10)
            .style(styles::button::choose_account)
            .on_press(Message::SelectRecipient(recipient_index).into())
    }

    fn selected_assets(
        recipient_index: usize,
        recipient: &'a Recipient,
        wallet: &'a Wallet<Unlocked>,
    ) -> Column<'a, AppMessage> {
        let mut assets: Vec<Element<'a, AppMessage>> =
            Vec::with_capacity(recipient.resources.len());

        for (resource_address, (symbol, amount)) in recipient.resources.iter() {
            let icon: Element<'a, AppMessage> = wallet
                .resource_icons()
                .get(&resource_address)
                .and_then(|bytes| {
                    Some(
                        widget::image(Handle::from_bytes(bytes.clone()))
                            .width(25)
                            .height(25)
                            .into(),
                    )
                })
                .unwrap_or(
                    container(text(Bootstrap::Image).font(BOOTSTRAP_FONT).size(18))
                        .center_x(25)
                        .center_y(25)
                        .into(),
                );

            let symbol = Self::resource_text_field(&symbol);

            let space = widget::Space::new(Length::Fill, 1);

            let amount = widget::text_input("Amount", &amount)
                .width(100)
                .style(styles::text_input::asset_amount)
                .on_input(move |input| {
                    Message::UpdateResourceAmount(recipient_index, resource_address.clone(), input)
                        .into()
                })
                .on_paste(move |input| {
                    Message::UpdateResourceAmount(recipient_index, resource_address.clone(), input)
                        .into()
                });

            let remove_resource = widget::button(
                text(Bootstrap::XLg)
                    .font(BOOTSTRAP_FONT)
                    .size(15)
                    .line_height(1.),
            )
            .padding(0)
            .style(button::text)
            .on_press(Message::RemoveAsset(recipient_index, resource_address.clone()).into());

            let resource_row = row![icon, symbol, space, amount, remove_resource]
                .spacing(10)
                .padding(5)
                .align_y(Alignment::Center)
                .height(Length::Shrink)
                .width(Length::Fill);

            assets.push(Rule::horizontal(1).into());
            assets.push(resource_row.into());
        }

        if assets.len() > 0 {
            assets.push(widget::Rule::horizontal(1).into());
        }

        widget::column(assets).spacing(1).width(Length::Fill)
    }
}
