use cursive::style::{BorderStyle, Palette};
use cursive::traits::*;
use cursive::views::Button;
use cursive::views::Dialog;
use cursive::views::LinearLayout;
use cursive::views::TextView;
use cursive::Cursive;
use cursive::CursiveExt;
use futures::executor::block_on;
use goldendict_ng_helper::favorite::{
    extract_all_words_from_favorites, extract_words_from_favorites_folder,
};
use goldendict_ng_helper::fsrs::sqlite_history::SQLiteHistory;
use rand::prelude::SliceRandom;
use rand::rng;
use rs_fsrs::Rating;
use shadow_rs::shadow;
use std::process::Command;
use urlencoding::encode;

shadow!(build);

static OCEAN: &str = "ocean";

#[tokio::main]
async fn main() {
    let mut history = SQLiteHistory::default().await;

    match std::env::args().nth(1).as_deref() {
        Some("--help") => {
            println!("used with goldendict-ng");
            println!("https://github.com/lengyijun/goldendict-ng-helper");
            println!("{}", build::VERSION); //print version const
            return;
        }
        Some("favourite") => {
            let favorite_words = extract_all_words_from_favorites().unwrap();
            let mut v = Vec::new();
            for word in favorite_words.into_iter() {
                if history.should_review(&word).await.is_ok() {
                    v.push(word);
                }
            }
            if v.is_empty() {
                println!("no words to review in favourite");
                return;
            }
            v.shuffle(&mut rng());
            history.queue.extend(v);
        }
        Some(folder_name) => {
            let favorite_words = extract_words_from_favorites_folder(folder_name).unwrap();
            let mut v = Vec::new();
            for word in favorite_words.into_iter() {
                if history.should_review(&word).await.is_ok() {
                    v.push(word);
                }
            }
            if v.is_empty() {
                println!("no words to review in folder {folder_name}");
                return;
            }
            v.shuffle(&mut rng());
            history.queue.extend(v);
        }
        None => {}
    }

    let Ok(word) = history.next_to_review().await else {
        println!("no words to review");
        return;
    };
    let mut siv = Cursive::default();
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

    siv.add_fullscreen_layer(
        Dialog::around(TextView::new(" ".repeat(200))) // move the title to center
            .title(word)
            .content(show_answer_layout())
            .h_align(cursive::align::HAlign::Center)
            .with_name(OCEAN),
        // .padding(Margins::lrtb(10, 10, 0, 35))
    );

    siv.run();

    let history: SQLiteHistory = siv.take_user_data().unwrap();
    println!("{:?}", history.history);
    println!("{:?}", history.queue);
}

fn show_answer_cb(s: &mut Cursive) {
    s.call_on_name(OCEAN, |view: &mut Dialog| {
        let word = view.get_title().to_owned();
        let url = format!("goldendict://{}", encode(&word));
        let _ = Command::new("xdg-open").arg(&url).status();

        let word_1 = word.clone();
        let word_2 = word.clone();
        let word_3 = word.clone();
        let word_4 = word;

        let buttons_layout = LinearLayout::horizontal()
            .child(Button::new("Skip", move |s| {
                review_next(s);
            }))
            .child(TextView::new(" ".repeat(100)))
            .child(Button::new("Again", move |s| {
                update_and_review_next(s, &word_1, Rating::Again);
            }))
            .child(TextView::new(" "))
            .child(Button::new("Hard", move |s| {
                update_and_review_next(s, &word_2, Rating::Hard);
            }))
            .child(TextView::new(" "))
            .child(Button::new("Good", move |s| {
                update_and_review_next(s, &word_3, Rating::Good);
            }))
            .child(TextView::new(" "))
            .child(Button::new("Easy", move |s| {
                update_and_review_next(s, &word_4, Rating::Easy);
            }))
            .child(TextView::new(" ".repeat(80)))
            .child(Button::new("Delete", move |s| {
                let word = s.call_on_name(OCEAN, |view: &mut Dialog| view.get_title().to_owned());
                if let Some(word) = word {
                    s.with_user_data(|history: &mut SQLiteHistory| block_on(history.delete(&word)));
                    review_next(s);
                }
            }))
            .child(Button::new("Quit", |s| {
                s.quit();
            }));

        view.set_content(buttons_layout);
    });
}

fn review_next(s: &mut Cursive) {
    let next_word =
        s.with_user_data(|history: &mut SQLiteHistory| block_on(history.next_to_review()));
    match next_word {
        Some(Ok(next_word)) => {
            s.call_on_name(OCEAN, |view: &mut Dialog| {
                view.set_title(next_word);
                view.set_content(show_answer_layout());
            });
        }
        _ => {
            s.quit();
        }
    }
}

fn update_and_review_next(s: &mut Cursive, word: &str, rating: Rating) {
    s.with_user_data(|history: &mut SQLiteHistory| {
        let _ = block_on(history.update(word, rating));
    });
    review_next(s);
}

fn show_answer_layout() -> LinearLayout {
    LinearLayout::horizontal()
        .child(Button::new("Skip", move |s| {
            review_next(s);
        }))
        .child(TextView::new(" ".repeat(100)))
        .child(Button::new("Show answer", show_answer_cb))
        .child(TextView::new(" ".repeat(100)))
        .child(Button::new("Quit", |s| {
            s.quit();
        }))
}
