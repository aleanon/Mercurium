use std::collections::{BTreeMap, HashMap};

use crate::{app::AppData, app::AppMessage, unlocked::app_view};
use font_and_icons::{Bootstrap, BOOTSTRAP_FONT};
use iced::{
    widget::{self, button, container, row, text, Container},
    Alignment, Element, Length, Padding, Task,
};
use ravault_iced_theme::styles;
use types::{Account, AccountAddress, Decimal, ResourceAddress};

use super::{
    add_assets::{self, AddAssets},
    add_recipient::{self, AddRecipient},
};

// pub struct TransactionView {
//     from_account: Option<String>,
//     recipient: Recipient,
//     message: Option<String>,
// }

const PICK_LIST_DEFAULT_TEXT: &'static str = "Select account";

#[derive(Debug, Clone)]
pub enum Message {
    OverView,
    SelectAccount(Account),
    UpdateMessage(String),
    RemoveRecipient(usize),
    UpdateResourceAmount(usize, ResourceAddress, String),
    SelectRecipient(usize),
    AddRecipient,
    ChooseRecipientMessage(add_recipient::Message),
    ///Pass the index of the account to add assets for
    AddAssets {
        recipient_index: usize,
        from_account: AccountAddress,
    },
    AddAssetsMessage(add_assets::Message),
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
pub struct TransactionView {
    pub(crate) from_account: Option<Account>,
    pub(crate) resource_amounts: HashMap<ResourceAddress, Decimal>,
    pub(crate) recipients: Vec<Recipient>,
    pub(crate) message: String,
    pub(crate) view: View,
}

impl TransactionView {
    pub fn new(
        from_account: Option<Account>,
        account_resources: Option<HashMap<ResourceAddress, Decimal>>,
    ) -> Self {
        Self {
            from_account,
            resource_amounts: account_resources.unwrap_or(HashMap::new()),
            recipients: vec![Recipient::new(None)],
            message: String::new(),
            view: View::Transaction,
        }
    }

    pub fn from_recipient(address: AccountAddress) -> Self {
        Self {
            from_account: None,
            resource_amounts: HashMap::new(),
            recipients: vec![Recipient::new(Some(address))],
            message: String::new(),
            view: View::Transaction,
        }
    }
}

impl<'a> TransactionView {
    pub fn update(&mut self, message: Message, appdata: &'a mut AppData) -> Task<AppMessage> {
        let mut command = Task::none();

        match message {
            Message::OverView => self.view = View::Transaction,
            Message::SelectAccount(account) => self.from_account = Some(account),
            Message::UpdateMessage(message) => self.message = message,
            Message::RemoveRecipient(recipient_index) => self.remove_recipient(recipient_index),
            Message::UpdateResourceAmount(account_index, resource, amount) => {
                self.update_resource_amount(account_index, resource, amount)
            }
            Message::SelectRecipient(recipient_index) => {
                self.view = View::ChooseRecipient(AddRecipient::new(recipient_index))
            }
            Message::AddRecipient => self.recipients.push(Recipient::new(None)),
            Message::AddAssets {
                recipient_index,
                from_account,
            } => self.create_new_add_assets_view(recipient_index, from_account),
            Message::AddAssetsMessage(add_assets_message) => {
                if let View::AddAssets(add_assets) = &mut self.view {
                    command = add_assets.update(add_assets_message, &mut self.recipients, appdata);
                }
            }
            Message::ChooseRecipientMessage(choose_recipient_message) => {
                if let View::ChooseRecipient(choose_recipient) = &mut self.view {
                    command = choose_recipient.update(
                        choose_recipient_message,
                        &mut self.recipients,
                        appdata,
                    )
                }
            }
            Message::RemoveAsset(recipient_index, resource_address) => {
                self.recipients[recipient_index]
                    .resources
                    .remove(&resource_address);
            }
        }

        command
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

    pub fn view(&'a self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        match &self.view {
            View::Transaction => self.overview(appdata),
            View::ChooseRecipient(choose_recipient) => choose_recipient.view(appdata),
            View::AddAssets(choose_assets) => choose_assets.view(appdata),
        }
    }

    fn overview(&'a self, appdata: &'a AppData) -> Element<'a, AppMessage> {
        let mut accounts = appdata
            .accounts
            .values()
            .map(|account| account)
            .collect::<Vec<&Account>>();

        // The accounts are sorted by ID
        accounts.sort_unstable_by(|a, b| a.cmp(b));

        let header = Self::header("Transaction");
        let space = widget::Space::new(Length::Fill, 50);

        let from_account_field = self.from_account_field(accounts);
        let space2 = widget::Space::new(Length::Fill, 30);

        let recipient_field = self.recipients(appdata);

        let space3 = widget::Space::new(Length::Fill, 20);

        let add_recipient = button("Add recipient")
            .width(Length::Fill)
            .height(Length::Shrink)
            .on_press(Message::AddRecipient.into());

        let space4 = widget::Space::new(Length::Fill, 50);

        let message_field = self.message();

        let fields = widget::column![
            header,
            space,
            from_account_field,
            space2,
            recipient_field,
            space3,
            add_recipient,
            space4,
            message_field
        ]
        .width(600)
        .height(Length::Shrink)
        .padding(Padding {
            left: 10.,
            right: 15.,
            ..Padding::ZERO
        });

        let row = row![
            widget::Space::new(Length::Fill, 1),
            fields,
            widget::Space::new(Length::Fill, 1)
        ];

        let scrollable = widget::scrollable(row).style(styles::scrollable::vertical_scrollable);

        // let left_space = widget::Space::new(Length::Fill, Length::Fill);
        // let right_space = widget::Space::new(Length::Fill, Length::Fill);

        // let content = widget::row![left_space, scrollable, right_space];
        widget::container(scrollable)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding {
                left: 5.,
                right: 5.,
                ..Padding::ZERO
            })
            .into()
    }

    fn message(&self) -> Container<'a, AppMessage> {
        let label = Self::field_label("Message");

        let text_field = widget::text_input("Enter Message", &self.message)
            .size(12)
            .line_height(1.5)
            .on_input(|message| Message::UpdateMessage(message).into())
            .on_paste(|message| Message::UpdateMessage(message).into())
            .style(styles::text_input::asset_amount);

        let rule = widget::Rule::horizontal(4).style(styles::rule::text_input_rule);

        let col = widget::column![label, text_field, rule].align_items(Alignment::Start);

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

    fn recipients(&'a self, appdata: &'a AppData) -> Container<'a, AppMessage> {
        let label = Self::field_label("TO");

        //create empty recipient

        let mut recipients: Vec<Element<'a, AppMessage>> =
            Vec::with_capacity(self.recipients.len());

        for (recipient_index, recipient) in self.recipients.iter().enumerate() {
            let choose_recipient_button = {
                let address = recipient
                    .address
                    .as_ref()
                    .and_then(|address| Some(address.truncate_long()))
                    .unwrap_or("Choose account".to_owned());

                let address = text(address).size(15).line_height(1.5).width(Length::Fill);

                let mut remove_recipient_button =
                    button(text(Bootstrap::XLg).font(BOOTSTRAP_FONT).line_height(1.))
                        .padding(0)
                        .style(styles::button::choose_account)
                        .on_press(Message::RemoveRecipient(recipient_index).into());

                if recipient_index == 0 {
                    if let None = &recipient.address {
                        remove_recipient_button = button(text("")).style(button::text);
                    }
                }

                let choose_recipient_content = row![address, remove_recipient_button]
                    .align_items(Alignment::Center)
                    .height(Length::Fill);

                button(choose_recipient_content)
                    .width(Length::Fill)
                    .height(50)
                    .padding(10)
                    .style(styles::button::choose_account)
                    .on_press(Message::SelectRecipient(recipient_index).into())
            };

            let assets = {
                let mut assets: Vec<Element<'a, AppMessage>> =
                    Vec::with_capacity(recipient.resources.len());

                for (resource_address, (symbol, amount)) in recipient.resources.iter() {
                    let icon: Element<'a, AppMessage> = appdata
                        .resource_icons
                        .get(&resource_address)
                        .and_then(|handle| {
                            Some(widget::image(handle.clone()).width(25).height(25).into())
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
                            Message::UpdateResourceAmount(
                                recipient_index,
                                resource_address.clone(),
                                input,
                            )
                            .into()
                        })
                        .on_paste(move |input| {
                            Message::UpdateResourceAmount(
                                recipient_index,
                                resource_address.clone(),
                                input,
                            )
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
                    .on_press(
                        Message::RemoveAsset(recipient_index, resource_address.clone()).into(),
                    );

                    let resource_row = row![icon, symbol, space, amount, remove_resource]
                        .spacing(10)
                        .padding(5)
                        .align_items(Alignment::Center)
                        .height(Length::Shrink)
                        .width(Length::Fill);

                    let resource = widget::column![widget::Rule::horizontal(1), resource_row]
                        .height(Length::Shrink)
                        .width(Length::Fill);

                    assets.push(resource.into())
                }

                widget::column(assets).spacing(1).width(Length::Fill)
            };

            let add_resource = widget::button(
                text("Add assets")
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                    .vertical_alignment(iced::alignment::Vertical::Center),
            )
            .height(30)
            .padding(0)
            .width(Length::Fill)
            .on_press_maybe(self.from_account.as_ref().and_then(|account| {
                Some(
                    Message::AddAssets {
                        recipient_index: recipient_index,
                        from_account: account.address.clone(),
                    }
                    .into(),
                )
            }));

            let recipient = widget::column![choose_recipient_button, assets, add_resource]
                .width(Length::Fill)
                .height(Length::Shrink);

            recipients.push(recipient.into())
        }

        let recipients = widget::column(recipients).spacing(10);

        widget::container(widget::column![label, recipients])
    }

    fn resource_text_field(str: &'a str) -> widget::Text<'a> {
        text(str).size(12).width(Length::Shrink)
    }

    fn header(input: &'a str) -> text::Text<'a> {
        text(input)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .size(20)
    }

    fn field_label(input: &'a str) -> text::Text<'a> {
        text(input)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .width(Length::Fill)
    }
}
