#[cfg(test)]
mod exclude_recoverable_tests {
    use std::{collections::BTreeSet, env::set_var};

    use crate::{tables::messages::Message, util::query_context::QueryContext};

    #[test]
    fn can_generate_filter_statement_empty() {
        let context = QueryContext::default();

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "")
    }

    #[test]
    fn can_generate_filter_statement_start() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(
            statement,
            " WHERE\n                     m.date >= 599558400000000000"
        )
    }

    #[test]
    fn can_generate_filter_statement_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_end("2020-01-01").unwrap();

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(
            statement,
            " WHERE\n                     m.date <= 599558400000000000"
        )
    }

    #[test]
    fn can_generate_filter_statement_start_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();
        context.set_end("2020-02-02").unwrap();

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, " WHERE\n                     m.date >= 599558400000000000 AND     m.date <= 602323200000000000")
    }

    #[test]
    fn can_generate_filter_statement_chat_ids() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(
            statement,
            " WHERE\n                     c.chat_id IN (1, 2, 3)"
        )
    }

    #[test]
    fn can_generate_filter_statement_start_end_chat_ids() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();
        context.set_end("2020-02-02").unwrap();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, " WHERE\n                     m.date >= 599558400000000000 AND     m.date <= 602323200000000000 AND     c.chat_id IN (1, 2, 3)")
    }

    #[test]
    fn can_create_invalid_start() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        assert!(context.set_start("2020-13-32").is_err());
        assert!(!context.has_filters());

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_create_invalid_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        assert!(context.set_end("fake").is_err());
        assert!(!context.has_filters());

        let statement = Message::generate_filter_statement(&context, false);
        assert_eq!(statement, "");
    }
}

#[cfg(test)]
mod include_recoverable_tests {
    use std::{collections::BTreeSet, env::set_var};

    use crate::{tables::messages::Message, util::query_context::QueryContext};

    #[test]
    fn can_generate_filter_statement_empty() {
        let context = QueryContext::default();

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, "")
    }

    #[test]
    fn can_generate_filter_statement_start() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(
            statement,
            " WHERE\n                     m.date >= 599558400000000000"
        )
    }

    #[test]
    fn can_generate_filter_statement_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_end("2020-01-01").unwrap();

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(
            statement,
            " WHERE\n                     m.date <= 599558400000000000"
        )
    }

    #[test]
    fn can_generate_filter_statement_start_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();
        context.set_end("2020-02-02").unwrap();

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, " WHERE\n                     m.date >= 599558400000000000 AND     m.date <= 602323200000000000")
    }

    #[test]
    fn can_generate_filter_statement_chat_ids() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(
            statement,
            " WHERE\n                     (c.chat_id IN (1, 2, 3) OR d.chat_id IN (1, 2, 3))"
        )
    }

    #[test]
    fn can_generate_filter_statement_start_end_chat_ids() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        context.set_start("2020-01-01").unwrap();
        context.set_end("2020-02-02").unwrap();
        context.set_selected_chat_ids(BTreeSet::from([1, 2, 3]));

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, " WHERE\n                     m.date >= 599558400000000000 AND     m.date <= 602323200000000000 AND     (c.chat_id IN (1, 2, 3) OR d.chat_id IN (1, 2, 3))")
    }

    #[test]
    fn can_create_invalid_start() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        assert!(context.set_start("2020-13-32").is_err());
        assert!(!context.has_filters());

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, "");
    }

    #[test]
    fn can_create_invalid_end() {
        set_var("TZ", "PST");

        let mut context = QueryContext::default();
        assert!(context.set_end("fake").is_err());
        assert!(!context.has_filters());

        let statement = Message::generate_filter_statement(&context, true);
        assert_eq!(statement, "");
    }
}

#[cfg(test)]
mod guid_query_tests {
    use std::env::current_dir;

    use crate::tables::{messages::Message, table::get_connection};

    #[test]
    fn test_cant_query_bad_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let message = Message::from_guid("fake-guid", &conn);

        assert!(message.is_err());
    }

    #[test]
    fn test_can_query_guid() {
        let db_path = current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("imessage-database/test_data/db/test.db");
        let conn = get_connection(&db_path).unwrap();

        let mut message =
            Message::from_guid("0355C6E1-D0C8-4212-AA87-DD8AE4FD1203", &conn).unwrap();
        let _ = message.generate_text(&conn);
        println!("{:#?}", message);
        assert!(message.components.is_some())
    }
}
