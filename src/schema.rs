table! {
    sessions (uuid) {
        uuid -> Uuid,
        user_id -> Int8,
        iat -> Timestamp,
        exp -> Timestamp,
    }
}

table! {
    users (id, username, discriminator) {
        id -> Int8,
        system -> Bool,
        bot -> Bool,
        username -> Text,
        password_hash -> Text,
        discriminator -> Varchar,
        bio -> Nullable<Text>,
        pronouns -> Nullable<Text>,
        avatar -> Nullable<Text>,
        banner -> Nullable<Text>,
        banner_colour -> Nullable<Varchar>,
        avatar_decoration -> Nullable<Text>,
        date_of_birth -> Nullable<Timestamp>,
        email -> Varchar,
        phone -> Nullable<Varchar>,
        mfa_enabled -> Bool,
        acct_verified -> Bool,
        flags -> Int4,
        nsfw_allowed -> Nullable<Bool>,
        purchased_flags -> Nullable<Int4>,
        premium_since -> Nullable<Timestamp>,
        premium_flags -> Nullable<Int4>,
        premium_type -> Nullable<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
