mod entry;
mod repo;

use std::collections::HashSet;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, bail};
use chrono_tz::Tz;
use clap::Parser;
use teloxide::prelude::*;
use teloxide::types::ReplyParameters;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use crate::entry::Entry;
use crate::repo::Repo;

const HELP: &str = "傳一則訊息就會在 vago 新增一篇 content 並 push。

格式：
YYYY-MM-DD HH:MM   ← 選填，預設用訊息時間
標題
內文（選填，Markdown）";

#[derive(Parser)]
#[command(
    name = "vago-bot",
    about = "收到 Telegram 訊息就在 vago 新增 content 並 push"
)]
struct Args {
    /// Telegram bot token
    #[arg(long, env = "TELEGRAM_BOT_TOKEN", hide_env_values = true)]
    token: String,

    /// 允許使用的人：數字 user ID 或 @username，逗號分隔
    #[arg(
        long,
        env = "VAGO_ALLOWED_USERS",
        value_delimiter = ',',
        required = true
    )]
    allowed_users: Vec<String>,

    /// vago repo 的 git remote
    #[arg(
        long,
        env = "VAGO_REPO_URL",
        default_value = "git@github.com:ckaznable/vago.git"
    )]
    repo_url: String,

    #[arg(long, env = "VAGO_BRANCH", default_value = "main")]
    branch: String,

    /// 本地 clone 的位置
    #[arg(long, env = "VAGO_REPO_DIR", default_value = "/app/data/vago")]
    repo_dir: PathBuf,

    /// 決定檔名與 year 的時區
    #[arg(long, env = "TZ", default_value = "Asia/Taipei")]
    tz: Tz,

    #[arg(long, env = "VAGO_GIT_NAME", default_value = "vago-bot")]
    git_name: String,

    #[arg(long, env = "VAGO_GIT_EMAIL", default_value = "vago-bot@axis.pi")]
    git_email: String,
}

struct State {
    repo: Mutex<Repo>,
    allowed_users: HashSet<String>,
    tz: Tz,
}

impl State {
    fn is_allowed(&self, msg: &Message) -> bool {
        let Some(user) = msg.from.as_ref() else {
            return false;
        };
        self.allowed_users.contains(&user.id.0.to_string())
            || user
                .username
                .as_ref()
                .is_some_and(|n| self.allowed_users.contains(&normalize_user(n)))
    }
}

/// 數字 ID 原樣保留，username 統一成小寫並加上 @
fn normalize_user(user: &str) -> String {
    let user = user.trim();
    if user.parse::<u64>().is_ok() {
        user.to_owned()
    } else {
        format!("@{}", user.trim_start_matches('@').to_lowercase())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_ansi(std::io::stdout().is_terminal())
        .init();

    let args = Args::parse();

    let allowed_users: HashSet<String> = args
        .allowed_users
        .iter()
        .filter(|u| !u.trim().is_empty())
        .map(|u| normalize_user(u))
        .collect();
    if allowed_users.is_empty() {
        bail!("VAGO_ALLOWED_USERS 不能是空的");
    }

    let repo = Repo {
        dir: args.repo_dir,
        url: args.repo_url,
        branch: args.branch,
        author_name: args.git_name,
        author_email: args.git_email,
    };
    repo.prepare().await?;
    info!(?allowed_users, "repo 已就緒，開始監聽 Telegram");

    let state = Arc::new(State {
        repo: Mutex::new(repo),
        allowed_users,
        tz: args.tz,
    });

    let mut dispatcher = Dispatcher::builder(
        Bot::new(args.token),
        Update::filter_message().endpoint(handle),
    )
    .dependencies(dptree::deps![state])
    .default_handler(|_| async {})
    .build();

    // k8s 停 Pod 送的是 SIGTERM，teloxide 內建的只處理 Ctrl-C
    let token = dispatcher.shutdown_token();
    tokio::spawn(async move {
        let mut term = signal(SignalKind::terminate()).expect("無法監聽 SIGTERM");
        tokio::select! {
            _ = term.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
        info!("收到停止訊號，等目前的訊息處理完後結束");
        if let Ok(done) = token.shutdown() {
            done.await;
        }
    });

    dispatcher.dispatch().await;
    Ok(())
}

async fn handle(bot: Bot, msg: Message, state: Arc<State>) -> ResponseResult<()> {
    if !state.is_allowed(&msg) {
        let user = msg.from.as_ref();
        warn!(
            id = user.map(|u| u.id.0),
            username = user.and_then(|u| u.username.as_deref()),
            "未授權的使用者，忽略訊息"
        );
        return Ok(());
    }

    let Some(text) = msg.text() else {
        return reply(&bot, &msg, "目前只支援文字訊息").await;
    };

    if text.starts_with('/') {
        return reply(&bot, &msg, HELP).await;
    }

    let now = msg.date.with_timezone(&state.tz).naive_local();
    let entry = match Entry::parse(text, now) {
        Ok(entry) => entry,
        Err(e) => return reply(&bot, &msg, &format!("❌ {e}\n\n{HELP}")).await,
    };

    let result = state.repo.lock().await.publish(&entry).await;
    match result {
        Ok(published) => {
            info!(path = published.path, commit = published.commit, "已推送");
            let text = format!("✅ 已新增 {}（{}）", published.path, published.commit);
            reply(&bot, &msg, &text).await
        }
        Err(e) => {
            error!("新增 content 失敗: {e:#}");
            reply(&bot, &msg, &format!("❌ 失敗：{e:#}")).await
        }
    }
}

async fn reply(bot: &Bot, msg: &Message, text: &str) -> ResponseResult<()> {
    bot.send_message(msg.chat.id, text)
        .reply_parameters(ReplyParameters::new(msg.id))
        .await?;
    Ok(())
}
