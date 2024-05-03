use bip39::Mnemonic;
use debug_print::debug_println;
use handles::EncryptedMnemonic;
use iced::futures::SinkExt;
use iced::Command;
use zeroize::Zeroize;

use crate::{
    app::AppData,
    message::{common_message::CommonMessage, Message},
    view::setup::{new_wallet::NewWalletStage, Setup},
    CREDENTIALS_STORE_NAME,
};
use store::Db;
use types::{crypto::Password, Action, Network};

use super::{NewWallet, SetupMessage};

const INVALID_PASSWORD_LENGTH: &str = "Password must be between 16 and 64 characters long";
const NON_ASCII_CHARACTERS: &str = "Password contains invalid characters";
const EMPTY_ACCOUNT_NAME: &str = "Account name can not be empty";
const MINIMUM_PASSWORD_LENGTH: usize = 16;
const MAXIMUM_PASSWORD_LENGTH: usize = 64;

#[derive(Debug, Clone)]
pub enum WalletMessage {
    // Back,
    UpdatePassword(String),
    SubmitPassword,
    UpdateVerificationPassword(String),
    VerifiPassword,
    UpdateAccName(String),
    SubmitAccName,
    SeedPhrase,
    VerifySeedPhrase,
    UpdateInputSeed((usize, Vec<String>)),
    Finalize,
}

impl Into<Message> for WalletMessage {
    fn into(self) -> Message {
        Message::Setup(SetupMessage::NewWalletMessage(self))
    }
}

impl<'a> WalletMessage {
    pub fn process(self, setup: &'a mut Setup, app_data: &'a mut AppData) -> Command<Message> {
        if let Setup::NewWallet(new_wallet_state) = setup {
            match self {
                // Self::Back => Self::move_to_previous_step(new_wallet_state),
                Self::UpdatePassword(mut input) => {
                    Self::update_password_input(&mut input, new_wallet_state)
                }
                Self::SubmitPassword => Self::submit_password(new_wallet_state),
                Self::UpdateVerificationPassword(input) => {
                    Self::update_verified_password_input(input, new_wallet_state)
                }
                Self::VerifiPassword => Self::verifi_password(new_wallet_state),
                Self::UpdateAccName(input) => {
                    Self::update_account_name_input(input, new_wallet_state)
                }
                Self::SubmitAccName => Self::submit_account_name(new_wallet_state),
                Self::SeedPhrase => Self::show_seed_phrase(new_wallet_state),
                Self::VerifySeedPhrase => Self::verify_seed_phrase(new_wallet_state),
                Self::UpdateInputSeed((index, words)) => {
                    Self::update_input_seed(index, words, new_wallet_state)
                }
                Self::Finalize => Self::create_wallet(new_wallet_state, app_data),
            }
        } else {
            unreachable!()
        }
    }

    // fn move_to_previous_step(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
    //     match new_wallet_state.stage {
    //         NewWalletStage::EnterPassword => {
    //             return Command::
    //         }
    //         NewWalletStage::VerifyPassword => {
    //             new_wallet_state.stage = NewWalletStage::EnterPassword;
    //             new_wallet_state.verify_password.clear();
    //             new_wallet_state.notification = "";
    //         }
    //         NewWalletStage::EnterAccountName => {
    //             new_wallet_state.stage = NewWalletStage::EnterPassword;
    //             new_wallet_state.password.clear();
    //             new_wallet_state.verify_password.clear();
    //             new_wallet_state.notification = "";
    //         }
    //         NewWalletStage::EnterSeedPhrase => {
    //             new_wallet_state.stage = NewWalletStage::EnterAccountName;
    //             new_wallet_state.mnemonic = None;
    //             new_wallet_state.notification = "";
    //         }
    //         NewWalletStage::ViewSeedPhrase => {
    //             new_wallet_state.stage = NewWalletStage::EnterAccountName;
    //             new_wallet_state.notification = "";
    //         }
    //         NewWalletStage::VerifySeedPhrase => {
    //             new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
    //             new_wallet_state.notification = "";
    //             new_wallet_state.seed_phrase = SeedPhrase::new();
    //         }
    //     }

    //     Command::none()
    // }

    fn update_password_input(
        input: &mut String,
        new_wallet_state: &'a mut NewWallet,
    ) -> Command<Message> {
        new_wallet_state.password.clear();
        new_wallet_state.password.push_str(input.as_str());
        input.zeroize();

        Command::none()
    }

    fn submit_password(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
        if !new_wallet_state.password.as_str().is_ascii() {
            new_wallet_state.notification = NON_ASCII_CHARACTERS
        } else if new_wallet_state.password.as_str().len() < MINIMUM_PASSWORD_LENGTH
            || new_wallet_state.password.as_str().len() > MAXIMUM_PASSWORD_LENGTH
        {
            new_wallet_state.notification = INVALID_PASSWORD_LENGTH
        } else {
            new_wallet_state.stage = NewWalletStage::VerifyPassword;
            new_wallet_state.notification = "";
        }
        Command::none()
    }

    fn update_verified_password_input(
        mut input: String,
        new_wallet_state: &'a mut NewWallet,
    ) -> Command<Message> {
        new_wallet_state.verify_password.clear();
        new_wallet_state.verify_password.push_str(input.as_str());
        input.zeroize();

        Command::none()
    }

    fn verifi_password(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
        if new_wallet_state.verify_password.as_str() == new_wallet_state.password.as_str() {
            new_wallet_state.stage = NewWalletStage::EnterAccountName;
            new_wallet_state.notification = "";
        } else {
            new_wallet_state.notification = "Password does not match";
        }

        Command::none()
    }

    fn update_account_name_input(
        input: String,
        new_wallet_state: &'a mut NewWallet,
    ) -> Command<Message> {
        new_wallet_state.account_name = input;

        Command::none()
    }

    fn submit_account_name(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
        if new_wallet_state.account_name.len() == 0 {
            new_wallet_state.notification = EMPTY_ACCOUNT_NAME;
        } else {
            match new_wallet_state.mnemonic {
                Some(_) => new_wallet_state.stage = NewWalletStage::ViewSeedPhrase,
                None => new_wallet_state.stage = NewWalletStage::EnterSeedPhrase,
            }
            new_wallet_state.notification = "";
        }

        Command::none()
    }

    fn show_seed_phrase(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
        new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
        new_wallet_state.notification = "";

        Command::none()
    }

    fn verify_seed_phrase(new_wallet_state: &'a mut NewWallet) -> Command<Message> {
        new_wallet_state.stage = NewWalletStage::VerifySeedPhrase;
        new_wallet_state.notification = "";

        Command::none()
    }

    fn update_input_seed(
        mut index: usize,
        words: Vec<String>,
        new_wallet_state: &'a mut NewWallet,
    ) -> Command<Message> {
        for mut word in words {
            new_wallet_state.seed_phrase.update_word(index, &word);
            word.zeroize();
            index += 1;
        }

        Command::none()
    }

    fn create_wallet(
        new_wallet_state: &'a mut NewWallet,
        app_data: &'a mut AppData,
    ) -> Command<Message> {
        if let None = new_wallet_state.mnemonic {
            let phrase = new_wallet_state.seed_phrase.phrase();
            let mnemonic = match Mnemonic::from_phrase(phrase.as_str(), bip39::Language::English) {
                Ok(mnemonic) => mnemonic,
                Err(_) => {
                    new_wallet_state.notification = "Invalid seed phrase";
                    return Command::none();
                }
            };
            new_wallet_state.mnemonic = Some(mnemonic)
        }

        let mnemonic = new_wallet_state
            .mnemonic
            .as_ref()
            .unwrap_or_else(|| unreachable!("{}:{} Mnemonic not found", module_path!(), line!()));

        //Encrypt and store mnemonic as credentials

        let (key, _salt) = match new_wallet_state.password.derive_new_db_encryption_key() {
            Ok((key, salt)) => (key, salt),
            Err(_) => {
                new_wallet_state.notification =
                    "Unable to create random value for key derivation, please try again";
                return Command::none();
            }
        };

        let mut db = Db::new().unwrap_or_else(|err| {
            unreachable!(
                "{}:{} Unable to create database: {err}",
                module_path!(),
                line!()
            )
        });

        let account = handles::wallet::create_account_from_mnemonic(
            mnemonic,
            0,
            0,
            new_wallet_state.account_name.clone(),
            Network::Mainnet,
        );

        debug_println!("Account created");

        if let Err(err) = db.upsert_account(&account) {
            debug_println!(
                "{}:{}: Unable to write account to database, error: {}",
                module_path!(),
                line!(),
                err
            );
            //TODO("Implement notification in app {err}")
        }

        debug_println!("Account stored in database");

        app_data.db = db;
        // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
        //     app.state = State::Error(AppError::Fatal(Box::new(err)))
        // }

        let mut sender_clone = app_data.backend_sender.clone();
        let key_clone = key.clone();

        let load_database = Command::perform(
            async move { sender_clone.send(Action::LoadDatabase(key_clone)).await },
            |result| {
                if let Err(err) = result {
                    debug_println!("Unable to send command to backend: {:?}", err);

                    //todo!("implement app notification");
                    Message::None
                } else {
                    Message::Common(CommonMessage::PerformLogin(key))
                }
            },
        );

        let password = new_wallet_state.password.clone();
        let mnemonic = new_wallet_state.mnemonic.take().unwrap();
        let save_mnemonic = Command::perform(
            async move {
                EncryptedMnemonic::new(&mnemonic, &password).and_then(|encrypted_mnemonic| {
                    encrypted_mnemonic.save_to_store(CREDENTIALS_STORE_NAME)
                })
            },
            |result| match result {
                Ok(_account_address) => Message::None,
                Err(err) => CommonMessage::Notify(format!("Unable to save mnemonic: {err}")).into(),
            },
        );

        Command::batch([load_database, save_mnemonic])
    }
}
