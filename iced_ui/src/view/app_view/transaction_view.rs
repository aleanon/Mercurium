pub mod add_assets;
pub mod choose_recipient;

use std::collections::{BTreeMap, HashMap};

use crate::{
    app::App,
    message::{app_view_message::transaction_message::TransactionMessage, Message},
};
use iced::{
    theme,
    widget::{self, button, container, row, text, Container},
    Alignment, Element, Length, Padding,
};
use ravault_iced_theme::styles::{self, rule::TextInputRule};
use types::{Account, AccountAddress, Decimal, ResourceAddress};

use self::{add_assets::AddAssets, choose_recipient::ChooseRecipient};

// pub struct TransactionView {
//     from_account: Option<String>,
//     recipient: Recipient,
//     message: Option<String>,
// }

const PICK_LIST_DEFAULT_TEXT: &'static str = "Select account";

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
    ChooseResource(AddAssets),
    ChooseRecipient(ChooseRecipient),
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
    pub fn view(&'a self, app: &'a App) -> Element<'a, Message> {
        match &self.view {
            View::Transaction => self.overview(app),
            View::ChooseRecipient(choose_recipient) => choose_recipient.view(app),
            View::ChooseResource(choose_assets) => choose_assets.view(app),
        }
    }

    fn overview(&'a self, app: &'a App) -> Element<'a, Message> {
        let accounts_map = app.app_data.db.get_accounts().unwrap_or(BTreeMap::new());

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

        let scrollable = widget::scrollable(row)
            .style(theme::Scrollable::custom(styles::scrollable::Scrollable));

        // let left_space = widget::Space::new(Length::Fill, Length::Fill);
        // let right_space = widget::Space::new(Length::Fill, Length::Fill);

        // let content = widget::row![left_space, scrollable, right_space];
        widget::container(scrollable)
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .padding(Padding {
                left: 5.,
                right: 5.,
                ..Padding::ZERO
            })
            .into()
    }

    fn message(&self, _app: &'a App) -> Container<'a, Message> {
        let label = Self::field_label("Message");

        let text_field = widget::text_input("Enter Message", &self.message)
            .size(12)
            .line_height(1.5)
            .on_input(|message| TransactionMessage::UpdateMessage(message).into())
            .on_paste(|message| TransactionMessage::UpdateMessage(message).into())
            .style(iced::theme::TextInput::Custom(Box::new(
                styles::text_input::AssetAmount,
            )));

        let rule = widget::Rule::horizontal(4).style(TextInputRule::style);

        let col = widget::column![label, text_field, rule].align_items(Alignment::Start);

        widget::container(col)
    }

    fn from_account_field(&'a self, accounts: Vec<Account>) -> Container<'a, Message> {
        let label = Self::field_label("From");

        let (_account_name, _account_address) = match self.from_account {
            Some(ref account) => (account.name.as_str(), account.address.truncate()),
            None => ("Select Account", String::with_capacity(0)),
        };

        let picklist = widget::pick_list(accounts, self.from_account.as_ref(), |account| {
            TransactionMessage::SelectAccount(account).into()
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

    fn recipients(&'a self, app: &'a App) -> Container<'a, Message> {
        let label = Self::field_label("TO");

        //create empty recipient

        let mut recipients: Vec<Element<'a, Message>> = Vec::with_capacity(self.recipients.len());

        for (recipient_index, recipient) in self.recipients.iter().enumerate() {
            let choose_recipient_button = {
                let address = recipient
                    .address
                    .as_ref()
                    .and_then(|address| Some(address.truncate_long()))
                    .unwrap_or("Choose account".to_owned());

                let address = text(address).size(15).line_height(1.5).width(Length::Fill);

                let mut remove_recipient_button = button(
                    text(iced_aw::Bootstrap::XLg)
                        .font(iced_aw::BOOTSTRAP_FONT)
                        .line_height(1.),
                )
                .padding(0)
                .style(theme::Button::custom(styles::button::ChooseAccount))
                .on_press(TransactionMessage::RemoveRecipient(recipient_index).into());

                if recipient_index == 0 {
                    if let None = &recipient.address {
                        remove_recipient_button = button(text("")).style(theme::Button::Text);
                    }
                }

                let choose_recipient_content = row![address, remove_recipient_button]
                    .align_items(Alignment::Center)
                    .height(Length::Fill);

                button(choose_recipient_content)
                    .width(Length::Fill)
                    .height(50)
                    .padding(10)
                    .style(theme::Button::custom(styles::button::ChooseAccount))
                    .on_press(TransactionMessage::SelectRecipient(recipient_index).into())
            };

            let assets = {
                let mut assets: Vec<Element<'a, Message>> =
                    Vec::with_capacity(recipient.resources.len());

                for (resource_address, (symbol, amount)) in recipient.resources.iter() {
                    let icon: Element<'a, Message> = app
                        .app_data
                        .resource_icons
                        .get(&resource_address)
                        .and_then(|handle| {
                            Some(widget::image(handle.clone()).width(25).height(25).into())
                        })
                        .unwrap_or(
                            container(
                                text(iced_aw::Bootstrap::Image)
                                    .font(iced_aw::BOOTSTRAP_FONT)
                                    .size(18),
                            )
                            .width(25)
                            .height(25)
                            .center_x()
                            .center_y()
                            .into(),
                        );

                    let symbol = Self::resource_text_field(&symbol);

                    let space = widget::Space::new(Length::Fill, 1);

                    let amount = widget::text_input("Amount", &amount)
                        .width(100)
                        .style(theme::TextInput::Custom(Box::new(
                            styles::text_input::AssetAmount,
                        )))
                        .on_input(move |input| {
                            TransactionMessage::UpdateResourceAmount(
                                recipient_index,
                                resource_address.clone(),
                                input,
                            )
                            .into()
                        })
                        .on_paste(move |input| {
                            TransactionMessage::UpdateResourceAmount(
                                recipient_index,
                                resource_address.clone(),
                                input,
                            )
                            .into()
                        });

                    let remove_resource = widget::button(
                        text(iced_aw::Bootstrap::XLg)
                            .font(iced_aw::BOOTSTRAP_FONT)
                            .size(15)
                            .line_height(1.),
                    )
                    .padding(0)
                    .style(theme::Button::Text)
                    .on_press(
                        TransactionMessage::RemoveAsset(recipient_index, resource_address.clone())
                            .into(),
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
                    TransactionMessage::AddAssets {
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

    fn resource_text_field(str: &str) -> widget::Text<'a> {
        text(str).size(12).width(Length::Shrink)
    }

    fn header(input: &str) -> text::Text<'a> {
        text(input)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .width(Length::Fill)
            .size(20)
    }

    fn field_label(input: &str) -> text::Text<'a> {
        text(input)
            .horizontal_alignment(iced::alignment::Horizontal::Left)
            .vertical_alignment(iced::alignment::Vertical::Center)
            .width(Length::Fill)
    }
}
