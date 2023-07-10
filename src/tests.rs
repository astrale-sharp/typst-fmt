use super::*;

macro_rules! test_snippet {
    (
        $test_name:ident,
        $snippet:expr,
        formatted = $expected:expr,
        config = $config:expr $(,)?
    ) => {
        #[test]
        fn $test_name() {
            let _ = env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init();
            let formatted = format($snippet, $config);
            println!("===");
            println!(
                "input: {:?}\nexpected: {:?}\nfound: {:?}",
                $snippet, $expected, formatted
            );
            similar_asserts::assert_eq!($expected, formatted);
            println!("===");
        }
    };
}

test_snippet!(
    unchanged,
    "#{ }",
    formatted = "#{ }",
    config = Config::default()
);

test_snippet!(
    one_space,
    "#{  }",
    formatted = "#{ }",
    config = Config::default()
);
test_snippet!(
    one_space_b,
    "#{   }",
    formatted = "#{ }",
    config = Config::default()
);
test_snippet!(
    two_line_max,
    "\n\n\n",
    formatted = "\n\n",
    config = Config::default()
);
test_snippet!(
    empty_func_call,
    "#f()",
    formatted = "#f()",
    config = Config::default()
);
test_snippet!(
    simple_func_call,
    "#f(1,2,3)",
    formatted = "#f(1, 2, 3)",
    config = Config::default()
);
test_snippet!(
    long_func_call,
    "#f(1,this_is_absurdly_loooooooooong,3)",
    formatted = "#f(\n  1,\n  this_is_absurdly_loooooooooong,\n  3,\n)",
    config = Config {
        max_line_length: 5,
        ..Default::default()
    }
);

test_snippet!(
    long_func_call_trailing,
    "#f(1,this_is_absurdly_loooooooooong,3,)",
    formatted = "#f(\n  1,\n  this_is_absurdly_loooooooooong,\n  3,\n)",
    config = Config {
        max_line_length: 5,
        ..Default::default()
    }
);