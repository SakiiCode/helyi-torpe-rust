#![allow(clippy::needless_return)]
use std::cmp;
use std::io::Cursor;

use ab_glyph::{FontRef, PxScale};
use dotenv::dotenv;
use imageproc::drawing::draw_text_mut;
use imageproc::image::{ImageBuffer, Rgb};

use poise::{serenity_prelude as serenity, CreateReply};
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use serenity::{CreateAttachment, GetMessages, ReactionType};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DG: [&str; 10] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

const POLL_CHARS: [char; 11] = ['🇦', '🇧', '🇨', '🇩', '🇪', '🇫', '🇬', '🇭', '🇮', '🇯', '🇰'];

const MINE_COUNT: u32 = 10;
const MAP_SIZE: usize = 9;

macro_rules! idx {
    // `()` indicates that the macro takes no argument.
    ($map:expr, $x:expr, $y:expr) => {
        $map[($x * (MAP_SIZE + 2) + $y) as usize]
    };
}

#[poise::command(slash_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "**A Helyi Törpe parancsai**\n```
/help                                  parancsok listája
/source                                link a forráskódhoz
/minesweeper                           aknakereső
/meme <szöveg>                         a legutóbb feltöltött képhez felirat
/poll <kérdés> <válasz1,válasz2,...>   szavazás```",
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn poll(
    ctx: Context<'_>,
    #[description = "kérdés"] question: String,
    #[description = "válasz1,..."] answers: String,
) -> Result<(), Error> {
    let mut reply = format!("__Szavazás: **{}**__\n", question);
    let answer_arr = answers.split(',').collect::<Vec<&str>>();
    let answer_size = cmp::min(answer_arr.len(), 11);
    for i in 0..answer_size {
        reply.push_str(format!("{}:{}\n", POLL_CHARS[i], answer_arr[i]).as_str());
    }

    let message = ctx.say(reply).await?;
    let msg = message.into_message().await?;

    for char in POLL_CHARS.iter().take(answer_size) {
        msg.react(ctx.http(), ReactionType::Unicode(char.to_string()))
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
async fn minesweeper(ctx: Context<'_>) -> Result<(), Error> {
    let mut map = [0; (MAP_SIZE + 2) * (MAP_SIZE + 2)];
    let mut rng = SmallRng::from_rng(rand::thread_rng()).unwrap();

    for _ in 0..MINE_COUNT {
        let mut x;
        let mut y;
        loop {
            x = 1 + (rng.next_u32() as usize % MAP_SIZE);
            y = 1 + (rng.next_u32() as usize % MAP_SIZE);
            if idx!(map, x, y) != 9 {
                break;
            }
        }

        idx!(map, x, y) = 9;

        for j in 0..=2 {
            for k in 0..=2 {
                let x2 = x + j - 1;
                let y2 = y + k - 1;
                if idx!(map, x2, y2) != 9 {
                    idx!(map, x2, y2) += 1;
                }
            }
        }
    }

    let mut txt = format!("\n{MINE_COUNT} akna van elrejtve\n");

    for i in 1..=MAP_SIZE {
        for j in 1..=MAP_SIZE {
            if idx!(map, i, j) != 9 {
                txt.push_str(format!("||  :{}:  ||  ", DG[idx!(map, i, j)]).as_str());
            } else {
                txt.push_str("||  :boom:  ||  ");
            }
        }
        txt.push('\n');
    }

    ctx.say(txt).await?;

    Ok(())
}

#[poise::command(slash_command)]
async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("https://github.com/SakiiCode/helyi-torpe-rust")
        .await?;
    Ok(())
}

async fn create_meme(url: &str, text: &str) -> Result<Vec<u8>, Error> {
    let font = FontRef::try_from_slice(include_bytes!("Anonymous_Pro.ttf")).unwrap();

    let scale = PxScale { x: 36.0, y: 36.0 };
    let letter_width = 36.0 / 1.8;
    let letter_height = 36.0;

    let bigw = 1000;

    let padding = 25;
    let txtwmax = bigw - 2 * padding;
    let chars_per_line = f64::floor(txtwmax as f64 / letter_width) as usize;

    let multiline = bwrap::wrap!(&text, chars_per_line);
    let lines_arr: Vec<&str> = multiline.split('\n').collect();

    let txth = letter_height as u32 * lines_arr.len() as u32;

    //BELSŐ KÉP MÉRETEI
    let desth = 480;
    let destw = bigw - 2 * padding;

    // BELSŐ KÉP HELYE
    let imgy = txth + 2 * padding;

    //NAGY KÉP MAGASSÁGA
    let bigh = imgy + desth + padding;

    let mut image = ImageBuffer::from_pixel(bigw, bigh, Rgb([255, 255, 255]));

    for (line_idx, line) in lines_arr.into_iter().enumerate() {
        draw_text_mut(
            &mut image,
            Rgb([0u8, 0u8, 0u8]),
            padding as i32,
            (padding as f32 + line_idx as f32 * letter_height) as i32,
            scale,
            &font,
            line,
        );
    }

    let downloaded: Vec<u8> = reqwest::get(url)
        .await?
        .bytes()
        .await?
        .iter()
        .map(|b| b.to_owned())
        .collect();

    let picture = imageproc::image::load_from_memory(&downloaded)?;
    let resized = picture
        .resize(
            destw,
            desth,
            imageproc::image::imageops::FilterType::CatmullRom,
        )
        .to_rgb8();

    //KÉP HELYE
    let imgx = bigw / 2 - resized.width() / 2;
    imageproc::image::imageops::overlay(&mut image, &resized, imgx as i64, imgy as i64);

    //let path = Path::new(&arg);
    //image.save(path).unwrap();
    let mut result_bytes: Vec<u8> = Vec::new();
    image.write_to(
        &mut Cursor::new(&mut result_bytes),
        imageproc::image::ImageFormat::Png,
    )?;
    Ok(result_bytes)
}

#[poise::command(slash_command)]
async fn meme(ctx: Context<'_>, #[description = "szöveg"] text: String) -> Result<(), Error> {
    let channel = match ctx.guild_channel().await {
        Some(ch) => ch,
        None => {
            ctx.reply("Ez a parancs még csak szerveren használható")
                .await?;
            return Ok(());
        }
    };

    let messages = channel
        .messages(ctx.http(), GetMessages::new().limit(20))
        .await?;

    let last_attachment = messages
        .iter()
        .rev()
        .filter(|msg| {
            if !msg.attachments.is_empty() {
                let last_filename = msg.attachments.last().unwrap().filename.to_lowercase();
                return last_filename.ends_with(".png") || last_filename.ends_with(".jpg");
            }
            return false;
        })
        .map(|msg| msg.attachments.last().unwrap())
        .last();

    let url = match last_attachment {
        Some(attachment) => attachment.url.clone(),
        None => {
            ctx.reply("Nem találtam képet :(").await?;
            return Ok(());
        }
    };

    let result_bytes = match create_meme(&url, &text).await {
        Ok(result) => result,
        Err(error) => {
            ctx.reply(format!("Hiba: {}", error)).await?;
            return Ok(());
        }
    };

    let builder =
        CreateReply::default().attachment(CreateAttachment::bytes(result_bytes, "result.png"));
    ctx.send(builder).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // This line loads the environment variables from the ".env" file.
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![minesweeper(), help(), poll(), source(), meme()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error building client");
    println!("Client built");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
