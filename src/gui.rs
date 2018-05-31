use ggez::{GameResult, Context};
use ggez::graphics;

pub struct GUI {
    score: i32,
    font: graphics::Font
}

impl GUI {
    pub fn new(ctx: &mut Context) -> GameResult<GUI> {
        let deja_vu = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18)?;

        Ok(GUI {
            score: 0,
            font: deja_vu
        })
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let score_string: String = "Score: ".to_owned() + &self.score.to_string();

        let score_display = graphics::Text::new(ctx, &score_string, &self.font)?;

        let score_dest = graphics::Point2::new(10.0, 10.0);
        graphics::draw(ctx, &score_display, score_dest, 0.0)?;

        Ok(())
    }
}