use clap::Subcommand;
use crate::AdminOptions;

#[derive(Debug, Subcommand)]
pub(crate) enum UsersCommands {
    Find {
        username: String,
        discriminator: Option<i32>
    },
    SetFlag {
        user_id: i64,
        flag: i64,
    },
    UnsetFlag {
        user_id: i64,
        flag: i64,
    },
    AddUser {
        username: String,
        email: String,
        password: String,
    },
    DeleteUser {
        user_id: i64,
    },
    SetPassword {
        user_id: i64,
        password: String,
    }
}

pub(crate) async fn users_commands(options: AdminOptions, users: UsersCommands) {
    match users {
        UsersCommands::Find { .. } => {}
        UsersCommands::SetFlag { .. } => {}
        UsersCommands::UnsetFlag { .. } => {}
        UsersCommands::AddUser { .. } => {}
        UsersCommands::DeleteUser { .. } => {}
        UsersCommands::SetPassword { .. } => {}
    }
}