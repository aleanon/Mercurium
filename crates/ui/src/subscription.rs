use backend::BackEnd;
use debug_print::debug_println;
use iced::{futures::SinkExt, Subscription};
use types::{Action, Update};

pub struct BackendWorker;

impl BackendWorker {
  pub fn backend_subscription() -> Subscription<Update> {
    iced::subscription::channel(
      std::any::TypeId::of::<Self>(),
      50,
      |mut output| async move {
          let (action_tx, action_rx) = iced::futures::channel::mpsc::channel(50);
          let mut backend = BackEnd::new(action_rx).unwrap();
          backend.load(&mut output).await.unwrap();

          if let Err(err) = output.send(Update::Sender(action_tx)).await {
              debug_println!(
                  "Module: {}, Line: {} Unable to send transmitter {err}",
                  module_path!(),
                  line!()
              )
              
          }

          loop {
              use iced::futures::StreamExt;
              let action = backend.action_rx.select_next_some().await;

              match action {
                  Action::LoadDatabase(key) => backend.action_load_db(&mut output).await,
                  Action::UpdateAll => backend.action_update_all(&mut output).await,
                  _ => {
                      output.send(Update::None).await.unwrap();
                  }
              }
          }
      },
    )
  }
}
