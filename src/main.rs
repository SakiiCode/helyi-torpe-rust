use std::cmp;

use ::serenity::all::{GuildId, ReactionType};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

const DG: [&str; 10] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
];

const POLL_CHARS: [char; 11] = ['🇦', '🇧', '🇨', '🇩', '🇪', '🇫', '🇬', '🇭', '🇮', '🇯', '🇰'];

const MINE_COUNT: u32 = 10;
const MAP_SIZE: u32 = 9;

macro_rules! idx {
    // `()` indicates that the macro takes no argument.
    ($map:expr, $x:expr, $y:expr) => {
        $map[($x * MAP_SIZE + $y) as usize]
    };
}

#[poise::command(slash_command, prefix_command)]
async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "**A Helyi Törpe parancsai**\n```
            #bot-spam
               t.help                          parancsok
               t.roles                         role-ok listája
               t.iam <szerep>                  role fel/levétele
               t.source                        a Helyi Törpe forráskódja
               t.minesweeper                   aknakereső
            bárhol
               t.meme <szöveg>                 legutóbbi képedhez felirat
               t.poll <kérdés,válasz1,...>     szavazás
               xd...                             xd```",
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn poll(
    ctx: Context<'_>,
    #[description = "kérdés"] question: String,
    #[description = "válasz1,..."] answers: String,
) -> Result<(), Error> {
    let mut reply = String::from(format!("__Szavazás: **{}**__\n", question));
    let answer_arr = answers.split(",").collect::<Vec<&str>>();
    let answer_size = cmp::min(answer_arr.len(), 11);
    for i in 0..answer_size {
        reply.push_str(format!("{}:{}\n", POLL_CHARS[i], answer_arr[i]).as_str());
    }

    let message = ctx.say(reply).await?;
    let msg = message.into_message().await?;

    for i in 0..answer_size {
        let char = POLL_CHARS[i];
        msg.react(ctx.http(), ReactionType::Unicode(char.to_string()))
            .await?;
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn minesweeper(ctx: Context<'_>) -> Result<(), Error> {
    let mut map = vec![0; (MAP_SIZE * MAP_SIZE) as usize];
    let mut rng = SmallRng::from_rng(rand::thread_rng()).unwrap();

    for _ in 0..MINE_COUNT {
        let mut x;
        let mut y;
        loop {
            x = rng.next_u32() % MAP_SIZE;
            y = rng.next_u32() % MAP_SIZE;
            if idx!(map, x, y) != 9 {
                break;
            }
        }

        map[(x * MAP_SIZE + y) as usize] = 9;

        for j in -1..=1 {
            for k in -1..=1 {
                let i_x: i32 = i32::try_from(x).unwrap() + j;
                let i_y: i32 = i32::try_from(y).unwrap() + k;

                if i_x > -1 && i_y > -1 {
                    let u_x: u32 = i_x.try_into().unwrap();
                    let u_y: u32 = i_y.try_into().unwrap();

                    if u_x < MAP_SIZE && u_y < MAP_SIZE && idx!(map, u_x, u_y) != 9 {
                        idx!(map, u_x, u_y) += 1;
                    }
                }
            }
        }
    }

    let mut txt = String::from(format!("{MINE_COUNT} akna van elrejtve (könnyű)"));

    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
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

#[poise::command(slash_command, prefix_command)]
async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("https://github.com/SakiiCode/helyi-torpe/blob/master/server.js")
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // This line loads the environment variables from the ".env" file.
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![minesweeper(), help(), poll(), source()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("t.".into()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
