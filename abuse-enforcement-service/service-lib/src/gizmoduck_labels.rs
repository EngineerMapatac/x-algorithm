pub fn label_ordinal_to_name(ordinal: i32) -> String {
    match ordinal {
        23 => "SpamHighRecall".to_owned(),
        other => format!("UnknownLabel({other})"),
    }
}
