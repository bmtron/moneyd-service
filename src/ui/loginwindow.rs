use cursive::Cursive;
use cursive::CursiveRunnable;
use cursive::style::{BorderStyle, Palette};

use cursive::traits::*;
use cursive::views::{Dialog, EditView, TextView};

use crate::utils::logintransporter::LoginRequest;
pub fn build_login_window() -> CursiveRunnable {
    let mut siv = cursive::default();
    siv.set_theme(cursive::theme::Theme {
        shadow: false,
        borders: cursive::theme::BorderStyle::Simple,
        palette: Palette::retro().with(|p| {
            use cursive::style::BaseColor::*;
            {
                use cursive::style::Color::TerminalDefault;
                use cursive::style::PaletteColor::*;
                p[Background] = TerminalDefault;
                p[View] = TerminalDefault;
                p[Primary] = White.dark();
                p[TitlePrimary] = Blue.light();
            }
        }),
    });
    siv.set_user_data(LoginRequest {
        email: String::new(),
        password: String::new(),
    });
    siv.add_layer(
        Dialog::new()
            .title("Enter your email:")
            .padding_lrtb(1, 1, 1, 0)
            .content(
                EditView::new()
                    .on_submit(submit_email)
                    .with_name("email")
                    .fixed_width(40),
            )
            .button("Ok", |s| {
                let email = s
                    .call_on_name("email", |view: &mut EditView| view.get_content())
                    .unwrap();

                submit_email(s, &email);
            }),
    );
    siv
}

fn submit_email(s: &mut Cursive, email: &str) {
    if email.is_empty() {
        s.add_layer(Dialog::info("Please enter a valid email address."));
    } else {
        s.with_user_data(|data: &mut LoginRequest| data.email = email.to_string());
        s.add_layer(
            Dialog::new()
                .title("Enter your password:")
                .padding_lrtb(1, 1, 1, 0)
                .content(
                    EditView::new()
                        .secret()
                        .on_submit(submit_pass)
                        .with_name("pass")
                        .fixed_width(40),
                )
                .button("Ok", |s| {
                    let pass = s
                        .call_on_name("pass", |view: &mut EditView| view.get_content())
                        .unwrap();

                    submit_pass(s, &pass);
                }),
        )
    }
}

fn submit_pass(s: &mut Cursive, pass: &str) {
    if pass.is_empty() {
        s.add_layer(Dialog::info("Please enter a password."));
    } else {
        s.with_user_data(|data: &mut LoginRequest| data.password = pass.to_string());
        let content = format!("Submitted.");
        s.pop_layer();
        s.add_layer(Dialog::around(TextView::new(content)).button("Quit", |s| s.quit()));
    }
}
