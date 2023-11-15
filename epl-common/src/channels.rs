use num_derive::FromPrimitive;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, FromPrimitive)]
#[repr(i32)]
pub enum ChannelTypes {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDM = 3,
    GuildCategory = 4,
    GuildAnnouncement = 5,
    AnnouncementThread = 10,
    PublicThread = 11,
    PrivateThread = 12,
    GuildStageVoice = 13,
    GuildDirectory = 14,
    GuildForum = 15,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(i32)]
pub enum VideoQualityModes {
    Auto = 1,
    Full = 2,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(i64)]
pub enum ChannelFlags {
    Pinned = 1 << 1,
    RequireTag = 1 << 4,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(i32)]
pub enum SortOrderTypes {
    LatestActivity = 0,
    CreationDate = 1,
}

#[derive(Serialize, Deserialize, Debug)]
#[repr(i32)]
pub enum ForumLayoutTypes {
    NotSet = 0,
    ListView = 1,
    GalleryView = 2,
}
