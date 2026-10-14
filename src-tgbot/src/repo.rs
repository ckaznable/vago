use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use tokio::process::Command;
use tracing::{info, warn};

use crate::entry::Entry;

const CONTENT_DIR: &str = "content";
const COMMIT_MESSAGE: &str = "chore: new content";
const GIT_TIMEOUT: Duration = Duration::from_secs(60);
const PUSH_ATTEMPTS: u32 = 3;

/// 本地的 vago clone，所有操作都透過 git CLI（認證交給 GIT_SSH_COMMAND）
pub struct Repo {
    pub dir: PathBuf,
    pub url: String,
    pub branch: String,
    pub author_name: String,
    pub author_email: String,
}

pub struct Published {
    pub path: String,
    pub commit: String,
}

impl Repo {
    /// 沒有 clone 過就 shallow clone，有的話對齊遠端
    pub async fn prepare(&self) -> Result<()> {
        if self.dir.join(".git").exists() {
            return self.sync().await;
        }

        info!(url = %self.url, dir = %self.dir.display(), "clone repo");
        tokio::fs::create_dir_all(&self.dir).await?;
        let mut cmd = Command::new("git");
        cmd.args(["clone", "--depth", "1", "--single-branch", "--branch"])
            .arg(&self.branch)
            .arg(&self.url)
            .arg(&self.dir);
        run("git clone", &mut cmd).await?;
        Ok(())
    }

    /// 新增一篇 content、commit 並 push；遠端被搶先更新時重新同步再試
    pub async fn publish(&self, entry: &Entry) -> Result<Published> {
        let refspec = format!("HEAD:refs/heads/{}", self.branch);
        let mut attempt = 1;
        loop {
            self.sync().await?;
            let path = self.write(entry).await?;
            self.git(&["add", "--", &path]).await?;
            self.git(&["commit", "-m", COMMIT_MESSAGE]).await?;

            match self.git(&["push", "origin", &refspec]).await {
                Ok(_) => {
                    let commit = self.git(&["rev-parse", "--short", "HEAD"]).await?;
                    return Ok(Published { path, commit });
                }
                Err(e) if attempt < PUSH_ATTEMPTS => {
                    warn!(attempt, "push 失敗，重新同步後再試: {e:#}");
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// 丟掉所有本地狀態（包含上次沒推成功的 commit），對齊遠端 branch
    async fn sync(&self) -> Result<()> {
        self.git(&["fetch", "--depth", "1", "origin", &self.branch])
            .await?;
        self.git(&["reset", "--hard", "FETCH_HEAD"]).await?;
        self.git(&["clean", "-fd"]).await?;
        Ok(())
    }

    /// 寫到 content/YYYY-MM-DD-HHmm.md，同一分鐘已有檔案就接 -2、-3…
    async fn write(&self, entry: &Entry) -> Result<String> {
        let dir = self.dir.join(CONTENT_DIR);
        tokio::fs::create_dir_all(&dir).await?;

        let stem = entry.file_stem();
        let name = (1..)
            .map(|n| match n {
                1 => format!("{stem}.md"),
                n => format!("{stem}-{n}.md"),
            })
            .find(|name| !dir.join(name).exists())
            .expect("檔名序號用完了");

        tokio::fs::write(dir.join(&name), entry.render()).await?;
        Ok(format!("{CONTENT_DIR}/{name}"))
    }

    async fn git(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.arg("-C")
            .arg(&self.dir)
            .arg("-c")
            .arg(format!("user.name={}", self.author_name))
            .arg("-c")
            .arg(format!("user.email={}", self.author_email))
            .args(args);
        run(&format!("git {}", args[0]), &mut cmd).await
    }
}

async fn run(name: &str, cmd: &mut Command) -> Result<String> {
    cmd.env("GIT_TERMINAL_PROMPT", "0").kill_on_drop(true);

    let output = tokio::time::timeout(GIT_TIMEOUT, cmd.output())
        .await
        .with_context(|| format!("{name} 逾時"))?
        .with_context(|| format!("無法執行 {name}"))?;

    if !output.status.success() {
        bail!(
            "{name} 失敗: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command as StdCommand;

    use chrono::NaiveDateTime;

    use super::*;

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = StdCommand::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    }

    /// 建一個 bare remote，裡面有一個帶 content/ 的初始 commit
    fn remote(root: &Path) -> String {
        let remote = root.join("remote.git");
        let seed = root.join("seed");
        git(
            root,
            &["init", "--bare", "-b", "main", remote.to_str().unwrap()],
        );
        git(
            root,
            &["clone", remote.to_str().unwrap(), seed.to_str().unwrap()],
        );
        std::fs::create_dir_all(seed.join("content")).unwrap();
        std::fs::write(seed.join("content/old.md"), "old\n").unwrap();
        git(&seed, &["add", "."]);
        git(&seed, &["commit", "-m", "init"]);
        git(&seed, &["push", "origin", "HEAD:main"]);
        remote.to_str().unwrap().to_owned()
    }

    fn entry(title: &str) -> Entry {
        Entry {
            time: NaiveDateTime::parse_from_str("2026-10-14 16:22", "%Y-%m-%d %H:%M").unwrap(),
            title: title.to_owned(),
            body: String::new(),
            data_rec: 50,
        }
    }

    #[tokio::test]
    async fn publishes_and_pushes_new_content() {
        let root = tempfile::tempdir().unwrap();
        let url = remote(root.path());
        let repo = Repo {
            dir: root.path().join("work"),
            url: url.clone(),
            branch: "main".to_owned(),
            author_name: "vago-bot".to_owned(),
            author_email: "vago-bot@axis.pi".to_owned(),
        };
        repo.prepare().await.unwrap();

        let first = repo.publish(&entry("a")).await.unwrap();
        assert_eq!(first.path, "content/2026-10-14-1622.md");

        // 別人在 bot 之後推了新 commit，bot 下一次要接在它後面
        let other = root.path().join("other");
        git(root.path(), &["clone", &url, other.to_str().unwrap()]);
        std::fs::write(other.join("content/manual.md"), "manual\n").unwrap();
        git(&other, &["add", "."]);
        git(&other, &["commit", "-m", "manual"]);
        git(&other, &["push", "origin", "HEAD:main"]);

        let second = repo.publish(&entry("b")).await.unwrap();
        assert_eq!(second.path, "content/2026-10-14-1622-2.md");

        let remote_dir = Path::new(&url);
        let log = git(remote_dir, &["log", "--format=%an|%s", "main"]);
        assert_eq!(
            log,
            "vago-bot|chore: new content\nt|manual\nvago-bot|chore: new content\nt|init"
        );
        let files = git(remote_dir, &["ls-tree", "--name-only", "main", "content/"]);
        assert_eq!(
            files,
            "content/2026-10-14-1622-2.md\ncontent/2026-10-14-1622.md\ncontent/manual.md\ncontent/old.md"
        );
        let shown = git(remote_dir, &["show", "main:content/2026-10-14-1622-2.md"]);
        assert!(shown.contains("title: \"b\""));
    }
}
