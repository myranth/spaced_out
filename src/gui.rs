use ggez::{GameResult, Context};
use ggez::graphics;

pub struct GUI {
    pub score: i32,
    pub money: i32,
    pub spaceout_charge: i32,
    font: graphics::Font
}

impl GUI {
    pub fn new(ctx: &mut Context) -> GameResult<GUI> {
        let deja_vu = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 18)?;

        Ok(GUI {
            score: 0,
            money: 100,
            spaceout_charge: 0,
            font: deja_vu
        })
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let score_string: String = "Score: ".to_owned() + &self.score.to_string();
        let money_string: String = format!("Money: {}", &self.money.to_string());
        let spaceout_charge_string = format!("Charge: {}", &self.spaceout_charge.to_string());

        let score_display = graphics::Text::new(ctx, &score_string, &self.font)?;
        let money_display = graphics::Text::new(ctx, &money_string, &self.font)?;
        let spaceout_charge_display = graphics::Text::new(ctx, &spaceout_charge_string, &self.font)?;

        let score_dest = graphics::Point2::new(10.0, 10.0);
        graphics::draw(ctx, &score_display, score_dest, 0.0)?;

        let money_dest = graphics::Point2::new(10.0, 30.0);
        graphics::draw(ctx, &money_display, money_dest, 0.0)?;

        let charge_dest = graphics::Point2::new(10.0, 50.0);
        graphics::draw(ctx, &spaceout_charge_display, charge_dest, 0.0)?;

        Ok(())
    }
}