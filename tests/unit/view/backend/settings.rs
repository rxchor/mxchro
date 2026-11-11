use super::run;
use crate::application::{
    dto::{AppSettingsDTO, LogoChoice, ThemeChoice},
    ports::{SettingsError, SettingsRepository},
};
use std::sync::{Arc, Mutex, mpsc};

struct Repository(Arc<Mutex<Vec<AppSettingsDTO>>>);
impl SettingsRepository for Repository {
    fn load(&self) -> Result<AppSettingsDTO, SettingsError> {
        Ok(Default::default())
    }
    fn save(&self, settings: AppSettingsDTO) -> Result<(), SettingsError> {
        self.0.lock().unwrap().push(settings);
        if settings.theme == ThemeChoice::Catppuccin {
            Err(SettingsError::Write)
        } else {
            Ok(())
        }
    }
}

#[test]
fn preference_worker_preserves_order_and_continues_after_a_failed_write() {
    let writes = Arc::new(Mutex::new(Vec::new()));
    let repository = Repository(writes.clone());
    let (requests, receiver) = mpsc::channel();
    let (sender, updates) = mpsc::channel();
    let worker = std::thread::spawn(move || run(&repository, receiver, sender));
    let rejected = AppSettingsDTO {
        theme: ThemeChoice::Catppuccin,
        logo: LogoChoice::Monochrome,
    };
    let accepted = AppSettingsDTO {
        theme: ThemeChoice::RosePine,
        logo: LogoChoice::FollowTheme,
    };
    requests.send(rejected).unwrap();
    requests.send(accepted).unwrap();
    drop(requests);
    worker.join().unwrap();
    assert_eq!(updates.try_recv(), Ok(Err(SettingsError::Write)));
    assert_eq!(updates.try_recv(), Ok(Ok(())));
    assert_eq!(*writes.lock().unwrap(), [rejected, accepted]);
}
