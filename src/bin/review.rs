#![feature(async_closure)]

use cursive::traits::*;
use cursive::views::Dialog;
use cursive::Cursive;
use cursive::CursiveExt;
use futures::executor::block_on;
use mdict_cli_rs::fsrs::sqlite_history::SQLiteHistory;
use mdict_cli_rs::spaced_repetition::SpacedRepetition;
use rs_fsrs::Rating;

#[tokio::main]
async fn main() {
    let mut siv = Cursive::default();
    let mut history = block_on(SQLiteHistory::default());
    let Ok(word) = block_on(history.next_to_review()) else {
        println!("no words to review");
        return;
    };
    siv.set_user_data(history);

    // Create a linear layout to arrange buttons horizontally at the bottom
    // let button_layout = LinearLayout::horizontal()
    //     .child(easy_btn)
    //     .child(good_btn)
    //     .child(hard_btn)
    //     .child(again_btn);

    // Place the buttons at the bottom of the screen
    siv.add_fullscreen_layer(
        Dialog::new()
            .title(word)
            .button("Show answer", |s| {
                s.add_layer(Dialog::info("Good selected"));
                s.call_on_name("ocean", |view: &mut Dialog| {
                    view.clear_buttons();

                    let word = view.get_title().to_owned();
                    view.add_button("Easy", move |s: &mut Cursive| {
                        update_and_review_next(s, word.clone(), Rating::Easy);
                    });

                    let word = view.get_title().to_owned();
                    view.add_button("Good", move |s| {
                        update_and_review_next(s, word.clone(), Rating::Good);
                    });

                    let word = view.get_title().to_owned();
                    view.add_button("Hard", move |s| {
                        update_and_review_next(s, word.clone(), Rating::Hard);
                    });

                    let word = view.get_title().to_owned();
                    view.add_button("Again", move |s| {
                        update_and_review_next(s, word.clone(), Rating::Again);
                    });
                });
            })
            .with_name("ocean"), // .content(button_layout)
                                 // .padding(Margins::lrtb(10, 10, 0, 35))
    );

    siv.run();
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
            });
        }
        _ => {
            s.quit();
        }
    }
}
