use binroots::binroots_enum;
use binroots::binroots_struct;
use binroots::save::SaveError;

#[binroots_enum]
enum Activity {
    None, // <- Automatically chosen as the default value thanks to #[binroots_enum]
    Playing(String),
}

#[binroots_struct] // <- Gives Status and its data the ability be saved to the disk
struct Status {
    connections: usize,
    is_online: bool,
    activity: Activity,
}

fn main() -> Result<(), SaveError> {
    let mut status = Status::default();

    *status.is_online = true;
    status.save()?; // <- Saves the entire struct to the disk

    *status.activity = Activity::Playing("video gamb".into());
    status.activity.save(Status::ROOT_FOLDER)?; // <- Only saves status.activity to the disk

    Ok(())
}
