
use bip39::Mnemonic;
use debug_print::debug_println;
use iced::Command;
use iced::futures::SinkExt;
use zeroize::Zeroize;

use crate::{
        app::{App, AppState}, message::Message, view::setup::{new_wallet::NewWalletStage, Setup} 
    };
use store::Db;
use types::{AppError, crypto::{Bip32Entity, Bip32KeyKind, Ed25519KeyPair, SeedPhrase}, Account, AccountAddress, Action, Network};


use super::{NewWallet, SetupMessage};

const INVALID_PASSWORD_LENGTH: &str = "Password must be between 16 and 64 characters long";
const NON_ASCII_CHARACTERS: &str = "Password contains invalid characters";
const EMPTY_ACCOUNT_NAME: &str = "Account name can not be empty";
const MINIMUM_PASSWORD_LENGTH:usize = 16;
const MAXIMUM_PASSWORD_LENGTH:usize = 64;

#[derive(Debug, Clone)]
pub enum WalletMessage {
    New,
    FromSeed,
    Back,
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
    pub fn process(self, app: &'a mut App) -> Command<Message> {
        match self {
            Self::New => Self::create_wallet_with_new_seed(app),
            Self::FromSeed => Self::create_wallet_user_supplied_seed(app),
            Self::Back => Self::move_to_previous_step(app),
            Self::UpdatePassword(mut input) => Self::update_password_input(&mut input, app),
            Self::SubmitPassword => Self::submit_password(app),
            Self::UpdateVerificationPassword(input) => Self::update_verified_password_input(input, app), 
            Self::VerifiPassword => Self::verifi_password(app),
            Self::UpdateAccName(input) => Self::update_account_name_input(input, app), 
            Self::SubmitAccName => Self::submit_account_name(app),
            Self::SeedPhrase => Self::show_seed_phrase(app),
            Self::VerifySeedPhrase => Self::verify_seed_phrase(app), 
            Self::UpdateInputSeed((index, words)) => Self::update_input_seed(index, words, app),
            Self::Finalize => Self::create_wallet(app),
        }
    }

    fn create_wallet_with_new_seed(app: &'a mut App) -> Command<Message> {
        app.app_state = AppState::Initial(Setup::NewWallet(NewWallet::new_with_mnemonic()));

        Command::none()
    }

    fn create_wallet_user_supplied_seed(app: &'a mut App) -> Command<Message> {
        app.app_state = AppState::Initial(Setup::NewWallet(NewWallet::new_without_mnemonic()));

        Command::none()
    }

    fn move_to_previous_step(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            match new_wallet_state.stage {
                NewWalletStage::EnterPassword => {
                    app.app_state = AppState::Initial(Setup::SelectCreation)
                }
                NewWalletStage::VerifyPassword => {
                    new_wallet_state.stage = NewWalletStage::EnterPassword;
                    new_wallet_state.verify_password.clear();
                    new_wallet_state.notification = "";
                }
                NewWalletStage::EnterAccountName => {
                    new_wallet_state.stage = NewWalletStage::EnterPassword;
                    new_wallet_state.password.clear();
                    new_wallet_state.verify_password.clear();
                    new_wallet_state.notification = "";
                }
                NewWalletStage::EnterSeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::EnterAccountName;
                    new_wallet_state.mnemonic = None;
                    new_wallet_state.notification = "";
                }
                NewWalletStage::ViewSeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::EnterAccountName;
                    new_wallet_state.notification = "";
                }
                NewWalletStage::VerifySeedPhrase => {
                    new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
                    new_wallet_state.notification = "";
                    new_wallet_state.seed_phrase = SeedPhrase::new();
                }
            }
        }       
        Command::none() 
    }

    fn update_password_input(input: &mut String, app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            new_wallet_state.password.clear();
            new_wallet_state.password.push_str(input.as_str());
            input.zeroize()
        }        
        Command::none()
    } 

    fn submit_password(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
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
        }
        Command::none()
    }

    fn update_verified_password_input(mut input: String, app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            new_wallet_state.verify_password.clear();
            new_wallet_state.verify_password.push_str(input.as_str());
            input.zeroize()
        }
        Command::none()
    }

    fn verifi_password(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            if new_wallet_state.verify_password.as_str()
                == new_wallet_state.password.as_str()
            {
                new_wallet_state.stage = NewWalletStage::EnterAccountName;
                new_wallet_state.notification = "";
            } else {
                new_wallet_state.notification = "Password does not match";
            }
        }
        Command::none() 
    }

    fn update_account_name_input(input: String, app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            new_wallet_state.account_name = input
        }
        Command::none()
    }

    fn submit_account_name(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            if new_wallet_state.account_name.len() == 0 {
                new_wallet_state.notification = EMPTY_ACCOUNT_NAME;
            } else {
                match new_wallet_state.mnemonic {
                    Some(_) => new_wallet_state.stage = NewWalletStage::ViewSeedPhrase,
                    None => new_wallet_state.stage = NewWalletStage::EnterSeedPhrase,
                }
                new_wallet_state.notification = "";
            }
        }
        Command::none()
    }

    fn show_seed_phrase(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            new_wallet_state.stage = NewWalletStage::ViewSeedPhrase;
            new_wallet_state.notification = "";
        }
        Command::none()
    }

    fn verify_seed_phrase(app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            new_wallet_state.stage = NewWalletStage::VerifySeedPhrase;
            new_wallet_state.notification = "";
        }
        Command::none()
    }

    fn update_input_seed(mut index: usize, words: Vec<String>, app: &'a mut App) -> Command<Message> {
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            for mut word in words {
                new_wallet_state.seed_phrase.update_word(index, &word);
                word.zeroize();
                index += 1;
            }
        }
        Command::none()
    }

    fn create_wallet(app: &'a mut App) -> Command<Message> {
        let mut command = Command::none();
        if let AppState::Initial(Setup::NewWallet(ref mut new_wallet_state)) = app.app_state {
            if let None = new_wallet_state.mnemonic {
                let phrase = new_wallet_state.seed_phrase.phrase();
                let mnemonic = match Mnemonic::from_phrase(
                    phrase.as_str(),
                    bip39::Language::English,
                ) {
                    Ok(mnemonic) => mnemonic,
                    Err(_) => {
                        new_wallet_state.notification = "Invalid seed phrase";
                        return Command::none();
                    }
                };
                new_wallet_state.mnemonic = Some(mnemonic)
            }

            let mnemonic = new_wallet_state.mnemonic.as_ref()
                .unwrap_or_else(|| unreachable!("{}:{} Mnemonic not found", module_path!(), line!()));

            //Encrypt and store mnemonic as credentials

            let (key, _salt) = match new_wallet_state.password.derive_new_db_encryption_key()
            {
                Ok((key, salt)) => (key, salt),
                Err(_) => {
                    new_wallet_state.notification = "Unable to create random value for key derivation, please try again";
                    return Command::none();
                }
            };

            let mut db = Db::new()
                .unwrap_or_else(|err| unreachable!("{}:{} Unable to create database: {err}", module_path!(), line!()));



            let account = handles::wallet::create_account_from_mnemonic(
                mnemonic, 0, new_wallet_state.account_name.clone(), Network::Mainnet
            );

            debug_println!("Account created");

            if let Err(err) = db.update_account(&account) {
                debug_println!(
                    "{}:{}: Unable to write account to database, error: {}",
                    module_path!(),
                    line!(),
                    err
                );
                //TODO("Implement notification in app {err}")
            }

            debug_println!("Account stored in database");

            app.db = Some(db);
            // if let Err(err) = app.action_tx.send(Action::LoadDatabase(key)) {
            //     app.state = State::Error(AppError::Fatal(Box::new(err)))
            // }

            if let Some(ref channel) = app.action_tx {
                command = {
                    let mut sender_clone = channel.clone();
                    Command::perform(
                        async move { sender_clone.send(Action::LoadDatabase(key)).await },
                        |result| {
                            if let Err(err) = result {
                                debug_println!(
                                    "Unable to send command to backend: {:?}",
                                    err
                                );

                                //todo!("implement app notification");
                                Message::None
                            } else {
                                Message::None
                            }
                        },
                    )
                };
            }

            if let Err(err) = app.login() {
                match err {
                    AppError::Fatal(_) => app.app_state = AppState::Error(err),
                    AppError::NonFatal(_err) => { /* impl app message*/ }
                }
            }
        }
        command
    }
}
