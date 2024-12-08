use cursive::style::{BorderStyle, Palette};
use cursive::traits::*;
use cursive::views::Dialog;
use cursive::views::TextView;
use cursive::Cursive;
use cursive::CursiveExt;
use futures::executor::block_on;
use goldendict_ng_helper::fsrs::sqlite_history::SQLiteHistory;
use goldendict_ng_helper::spaced_repetition::SpacedRepetition;
use rs_fsrs::Rating;
use shadow_rs::shadow;
use std::process::Command;
use urlencoding::encode;

shadow!(build);

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--help") {
        println!("used with goldendict-ng");
        println!("https://github.com/lengyijun/goldendict-ng-helper");
        println!("{}", build::VERSION); //print version const
        return;
    }

    let mut siv = Cursive::default();
    let mut history = SQLiteHistory::default().await;
    let Ok(word) = history.next_to_review().await else {
        println!("no words to review");
        return;
    };
    siv.set_user_data(history);

    // Start with a nicer theme than default
    siv.set_theme(cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Simple,
        palette: Palette::retro().with(|palette| {
            use cursive::style::BaseColor::*;

            {
                // First, override some colors from the base palette.
                use cursive::style::Color::TerminalDefault;
                use cursive::style::PaletteColor::*;

                palette[Background] = TerminalDefault;
                palette[View] = TerminalDefault;
                palette[Primary] = White.dark();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
            }

            {
                // Then override some styles.
                use cursive::style::Effect::*;
                use cursive::style::PaletteStyle::*;
                use cursive::style::Style;
                palette[Highlight] = Style::from(Blue.light()).combine(Bold);
                palette[EditableTextCursor] = Style::secondary().combine(Reverse).combine(Underline)
            }
        }),
    });

    // Create a linear layout to arrange buttons horizontally at the bottom
    // let button_layout = LinearLayout::horizontal()
    //     .child(easy_btn)
    //     .child(good_btn)
    //     .child(hard_btn)
    //     .child(again_btn);

    siv.add_fullscreen_layer(
        Dialog::around(TextView::new(" ".repeat(200))) // move the title to center
            .title(word)
            .button("Show answer", show_answer_cb)
            .h_align(cursive::align::HAlign::Center)
            .with_name("ocean"), // .content(button_layout)
                                 // .padding(Margins::lrtb(10, 10, 0, 35))
    );

    siv.run();
}

fn show_answer_cb(s: &mut Cursive) {
    s.call_on_name("ocean", |view: &mut Dialog| {
        view.clear_buttons();

        let word = view.get_title().to_owned();
        let url = format!("goldendict://{}", encode(&word));
        let _ = Command::new("xdg-open").arg(&url).status();

        let word = view.get_title().to_owned();
        view.add_button("Again", move |s| {
            update_and_review_next(s, word.clone(), Rating::Again);
        });

        let word = view.get_title().to_owned();
        view.add_button("Hard", move |s| {
            update_and_review_next(s, word.clone(), Rating::Hard);
        });

        let word = view.get_title().to_owned();
        view.add_button("Good", move |s| {
            update_and_review_next(s, word.clone(), Rating::Good);
        });

        let word = view.get_title().to_owned();
        view.add_button("Easy", move |s: &mut Cursive| {
            update_and_review_next(s, word.clone(), Rating::Easy);
        });
    });
}

fn update_and_review_next(s: &mut Cursive, word: String, rating: Rating) {
    let next_word = s.with_user_data(|history: &mut SQLiteHistory| {
        let _ = block_on(history.update(&word, rating));
        block_on(history.next_to_review())
    });
    match next_word {
        Some(Ok(next_word)) => {
            s.call_on_name("ocean", |view: &mut Dialog| {
                view.set_title(next_word);
                view.clear_buttons();
                view.add_button("Show answer", show_answer_cb);
            });
        }
        _ => {
            s.quit();
        }
    }
}
