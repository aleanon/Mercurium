use deps::{
    iced::{
        alignment::Horizontal,
        widget::{Button, Column, column},
    },
    *,
};

use std::collections::HashMap;

use crate::{
    app::AppMessage,
    components::{self, text_field::TextField},
    styles,
    unlocked::app_view,
};
use font_and_icons::{BOOTSTRAP_FONT, Bootstrap};
use iced::{
    Alignment, Element, Length, Padding, Task,
    widget::{self, Container, button, container, image::Handle, row, text},
};
use types::{
    Account, AppError, Decimal, Notification,
    address::{AccountAddress, Address, ResourceAddress},
    crypto::Password,
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
    ToggleTextField,
    InputPassword(String),
    SubmitTransaction,
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
    pub(crate) _resource_amounts: HashMap<ResourceAddress, Decimal>,
    pub(crate) recipients: Vec<Recipient>,
    pub(crate) text_field: Option<components::text_field::TextField>,
    pub(crate) view: View,
    /// Login password, entered at send time to authorize signing. Held only until the
    /// transaction is submitted (the mnemonic is re-decrypted per send, never cached).
    pub(crate) password: Password,
}

impl CreateTransaction {
    pub fn new(
        from_account: Option<Account>,
        account_resources: Option<HashMap<ResourceAddress, Decimal>>,
    ) -> Self {
        Self {
            from_account,
            _resource_amounts: account_resources.unwrap_or(HashMap::new()),
            recipients: vec![Recipient::new(None)],
            text_field: None,
            view: View::Transaction,
            password: Password::new(),
        }
    }

    pub fn from_recipient(address: AccountAddress) -> Self {
        Self {
            from_account: None,
            _resource_amounts: HashMap::new(),
            recipients: vec![Recipient::new(Some(address))],
            text_field: None,
            view: View::Transaction,
            password: Password::new(),
        }
    }

    /// Maps the in-memory recipient/asset model into a backend [`TransferRequest`].
    ///
    /// Returns `None` if the form is incomplete (no sender, a recipient without an address, an
    /// unparseable amount, or no assets to send). The resulting request is what gets passed to
    /// [`wallet::transaction::submit_transfer_with_password`] once the user confirms with their
    /// password.
    pub fn build_transfer_request(&self) -> Option<wallet::transaction::TransferRequest> {
        use std::str::FromStr;
        use types::RadixDecimal;
        use wallet::transaction::{FungibleTransfer, RecipientTransfer, TransferRequest};

        let from = self.from_account.as_ref()?.address.clone();

        let mut transfers = Vec::new();
        for recipient in &self.recipients {
            let Some(to) = recipient.address.clone() else {
                continue;
            };

            let mut fungibles = Vec::new();
            for (resource, (_symbol, amount)) in &recipient.resources {
                if amount.is_empty() {
                    continue;
                }
                // Reject the whole request if any amount is malformed rather than silently dropping it.
                let amount = RadixDecimal::from_str(amount).ok()?;
                fungibles.push(FungibleTransfer {
                    resource: resource.clone(),
                    amount,
                });
            }

            if fungibles.is_empty() {
                continue;
            }
            transfers.push(RecipientTransfer {
                to,
                fungibles,
                non_fungibles: Vec::new(),
            });
        }

        if transfers.is_empty() {
            return None;
        }

        Some(TransferRequest { from, transfers })
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
            Message::TextFieldMessage(message) => {
                self.text_field
                    .as_mut()
                    .and_then(|text_field| Some(text_field.update(message)));
            }
            Message::ToggleTextField => {
                self.text_field = match self.text_field {
                    Some(_) => None,
                    None => Some(TextField::new()),
                }
            }
            Message::InputPassword(input) => {
                self.password.clear();
                self.password.push_str(input.as_str());
            }
            Message::SubmitTransaction => return self.submit_transaction(wallet),
        }

        Task::none()
    }

    /// Builds the transfer request from the form, looks up the sending account, and spawns the
    /// async submit. The result is surfaced to the user via the app-level notification banner.
    fn submit_transaction(&mut self, wallet: &mut Wallet<Unlocked>) -> Task<AppMessage> {
        let Some(request) = self.build_transfer_request() else {
            return Task::done(AppMessage::Error(AppError::NonFatal(Notification::Info(
                "Add a sender, a recipient and at least one valid amount".to_string(),
            ))));
        };

        let Some(from_account) = wallet.accounts().get(&request.from).cloned() else {
            return Task::done(AppMessage::Error(AppError::NonFatal(Notification::Info(
                "Sending account is not in this wallet".to_string(),
            ))));
        };

        // Move the password out so it is dropped (zeroized) once the submit completes.
        let password = std::mem::replace(&mut self.password, Password::new());

        let env = wallet.env();
        let gateway = env.gateway(from_account.network);
        let secrets = env.secrets.clone();

        Task::perform(
            wallet::transaction::submit_transfer_with_password(
                request,
                from_account,
                gateway,
                secrets,
                password,
                0,
            ),
            |result| match result {
                Ok(submitted) => AppMessage::Error(AppError::NonFatal(Notification::Info(
                    format!("Transaction submitted: {}", submitted.intent_hash_id),
                ))),
                Err(err) => AppMessage::Error(err),
            },
        )
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

        accounts.sort_unstable_by(|a, b| a.id.cmp(&b.id));

        let header = Self::header("Transaction");

        let from_account_field = self.from_account_field(accounts);

        let recipient_field = self.recipients(wallet);

        let message_field = self.message();

        let fields = widget::column![header, from_account_field, recipient_field, message_field]
            .spacing(30)
            .height(Length::Shrink)
            .padding(Padding {
                left: 10.,
                right: 15.,
                ..Padding::ZERO
            });

        let scrollable =
            widget::scrollable(fields).style(styles::scrollable::vertical_scrollable_secondary);

        let page_top = widget::container(scrollable)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding {
                left: 5.,
                right: 5.,
                bottom: 10.,
                ..Padding::ZERO
            });

        let password_input = widget::text_input("Password to sign", self.password.as_str())
            .secure(true)
            .padding(10)
            .style(styles::text_input::base_layer_1_rounded)
            .on_input(|input| Message::InputPassword(input).into())
            .on_submit(Message::SubmitTransaction.into());

        // Enabled only once a sender is chosen and a password has been entered.
        let can_submit = self.from_account.is_some() && !self.password.is_empty();

        let create_transaction = container(
            column![
                self.review_summary(),
                password_input,
                button(text("Create transaction").width(Length::Fill).center())
                    .width(Length::Fill)
                    .style(styles::button::primary)
                    .height(50)
                    .on_press_maybe(can_submit.then(|| Message::SubmitTransaction.into())),
            ]
            .spacing(10),
        )
        .padding(Padding {
            left: 15.,
            right: 20.,
            ..Padding::ZERO
        });

        column![page_top, create_transaction].into()
    }

    /// A compact, human-readable review of what this transaction will send, shown above the
    /// password field so the user can confirm the details before signing. Renders nothing until
    /// at least one recipient has an address and an asset amount, mirroring the gate in
    /// [`build_transfer_request`].
    fn review_summary(&'a self) -> Element<'a, AppMessage> {
        let mut lines: Vec<Element<'a, AppMessage>> = Vec::new();

        if let Some(from) = &self.from_account {
            lines.push(
                row![
                    text("From").size(11).width(Length::Fixed(60.)).style(styles::text::muted),
                    text(format!("{} ({})", from.name, from.address.truncate())).size(12),
                ]
                .spacing(8)
                .into(),
            );
        }

        for recipient in &self.recipients {
            let Some(to) = &recipient.address else { continue };
            let assets: Vec<_> = recipient
                .resources
                .values()
                .filter(|(_symbol, amount)| !amount.is_empty())
                .collect();
            if assets.is_empty() {
                continue;
            }
            lines.push(
                row![
                    text("To").size(11).width(Length::Fixed(60.)).style(styles::text::muted),
                    text(to.truncate()).size(12),
                ]
                .spacing(8)
                .into(),
            );
            for (symbol, amount) in assets {
                lines.push(
                    row![
                        text("").width(Length::Fixed(60.)),
                        text(format!("{} {}", amount, symbol)).size(12),
                    ]
                    .spacing(8)
                    .into(),
                );
            }
        }

        // Nothing complete enough to review yet.
        if lines.len() <= 1 {
            return widget::column![].into();
        }

        let body = column![text("Review").size(13).style(styles::text::muted)]
            .extend(lines)
            .spacing(6);
        container(body)
            .padding(12)
            .width(Length::Fill)
            .style(styles::container::weak_layer_2_rounded_with_shadow)
            .into()
    }

    fn message(&'a self) -> Container<'a, AppMessage> {
        let label = Self::field_label("Message");

        let toggle = widget::Toggler::new(self.text_field.is_some())
            .size(20)
            .style(styles::toggler::base_layer_1)
            .on_toggle(|_| Message::ToggleTextField.into());

        let label_and_toggler = row![label, toggle].width(Length::Fill);

        let text_field = self.text_field.as_ref().and_then(|text_field_state| {
            Some(
                text_field_state
                    .view(|m| Message::TextFieldMessage(m).into())
                    .placeholder("Enter Message")
                    .padding(10)
                    .style(styles::text_editor::primary)
                    .height(120),
            )
        });

        let col = widget::column![label_and_toggler, text_field]
            .spacing(5)
            .align_x(Alignment::Start);

        // if let Some(text_field) = text_field {
        //     col = col.push(text_field);
        // }

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
        .menu_style(styles::menu::primary)
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
        let label = Self::field_label("To");

        let recipients = self
            .recipients
            .iter()
            .enumerate()
            .map(|(recipient_index, recipient)| self.recipient(recipient_index, recipient, wallet));

        let recipients = column(recipients).spacing(20);

        let add_recipient = row![
            widget::space().width(Length::FillPortion(2)),
            button(text("Add recipient").center())
                .padding(5)
                .width(Length::FillPortion(6))
                .height(Length::Shrink)
                .style(styles::button::base_layer_2_rounded_with_shadow)
                .on_press(Message::AddRecipient.into()),
            widget::space().width(Length::FillPortion(2))
        ];

        container(
            column![
                label,
                recipients,
                widget::space::horizontal().height(30),
                add_recipient
            ]
            .spacing(5),
        )
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
            .style(styles::text::muted)
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
        .style(styles::button::base_layer_2_rounded_with_shadow)
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
            .style(styles::container::weak_layer_2_rounded_with_shadow)
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
            .unwrap_or("Choose recipient".to_owned());

        let address = text(address).size(15).line_height(1.5).width(Length::Fill);

        let remove_recipient_button = if recipient_index == 0 && recipient.address.is_none() {
            button(text("")).style(button::text)
        } else {
            button(text(Bootstrap::XLg).font(BOOTSTRAP_FONT).line_height(1.))
                .padding(0)
                .style(styles::button::base_layer_2_rounded_with_shadow)
                .on_press(Message::RemoveRecipient(recipient_index).into())
        };

        let choose_recipient_content = row![address, remove_recipient_button]
            .align_y(Alignment::Center)
            .height(Length::Fill);

        button(choose_recipient_content)
            .height(50)
            .padding(10)
            .style(styles::button::base_layer_2_rounded_with_shadow)
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

            let space = widget::space::horizontal();

            let amount = widget::text_input("Amount", &amount)
                .width(100)
                .style(styles::text_input::base_layer_1_rounded)
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

            assets.push(widget::rule::horizontal(1).into());
            assets.push(resource_row.into());
        }

        if assets.len() > 0 {
            assets.push(widget::rule::horizontal(1).into());
        }

        widget::Column::from_vec(assets)
            .spacing(1)
            .width(Length::Fill)
    }
}
