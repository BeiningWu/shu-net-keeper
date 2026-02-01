use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

const MAX_LOG_LINES: usize = 1024;

/// Writer 包装类型，用于 tracing subscriber
/// 使用 Arc 共享所有权，允许跨线程使用
#[derive(Clone)]
struct WriterWrapper(Arc<LimitedLineWriter>);

impl Write for WriterWrapper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // 调用 &LimitedLineWriter 的 write 实现
        (&*self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        // 调用 &LimitedLineWriter 的 flush 实现
        (&*self.0).flush()
    }
}

/// 带行数限制的日志 Writer
/// 每次写入后检查，如果超过 MAX_LOG_LINES 行，则只保留最新的行
pub struct LimitedLineWriter {
    file: Mutex<File>,
    path: PathBuf,
    line_count: Mutex<usize>,
    write_count: Mutex<usize>, // 记录写入次数，每 N 次检查一次
}

impl LimitedLineWriter {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        // 如果文件存在，先检查并截断
        let initial_line_count = if path.exists() {
            let count = count_lines(&path)?;
            if count > MAX_LOG_LINES {
                truncate_to_last_n_lines(&path, MAX_LOG_LINES)?;
                MAX_LOG_LINES
            } else {
                count
            }
        } else {
            0
        };

        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            file: Mutex::new(file),
            path,
            line_count: Mutex::new(initial_line_count),
            write_count: Mutex::new(0),
        })
    }

    fn check_and_truncate(&self) -> io::Result<()> {
        let line_count = *self.line_count.lock().unwrap();

        if line_count > MAX_LOG_LINES {
            // 截断文件，只保留最后 MAX_LOG_LINES 行
            truncate_to_last_n_lines(&self.path, MAX_LOG_LINES)?;

            // 重新打开文件
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;

            *self.file.lock().unwrap() = new_file;
            *self.line_count.lock().unwrap() = MAX_LOG_LINES;
        }

        Ok(())
    }
}

impl Write for &LimitedLineWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut file = self.file.lock().unwrap();
        let n = file.write(buf)?;
        file.flush()?;

        // 统计新增的行数
        let new_lines = buf.iter().filter(|&&b| b == b'\n').count();
        *self.line_count.lock().unwrap() += new_lines;

        // 每 10 次写入检查一次
        let mut write_count = self.write_count.lock().unwrap();
        *write_count += 1;
        if *write_count >= 10 {
            *write_count = 0;
            drop(file); // 释放锁
            drop(write_count);
            self.check_and_truncate()?;
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.lock().unwrap().flush()
    }
}

/// 统计文件行数
fn count_lines(path: &PathBuf) -> io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

/// 截断文件，只保留最后 n 行
fn truncate_to_last_n_lines(path: &PathBuf, n: usize) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // 读取所有行
    let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>()?;

    // 只保留最后 n 行
    let keep_lines = if lines.len() > n {
        &lines[lines.len() - n..]
    } else {
        &lines
    };

    // 重新写入文件
    let mut file = File::create(path)?;
    for line in keep_lines {
        writeln!(file, "{}", line)?;
    }
    file.flush()?;

    Ok(())
}

/// 获取日志文件路径
fn get_log_path() -> io::Result<PathBuf> {
    let log_dir = match env::current_exe() {
        Ok(exe_path) => {
            // 在可执行文件目录下创建 logs 文件夹
            exe_path
                .parent()
                .map(|p| p.join("logs"))
                .unwrap_or_else(|| PathBuf::from("logs"))
        }
        Err(_) => PathBuf::from("logs"),
    };

    // 创建目录
    fs::create_dir_all(&log_dir)?;

    // 固定文件名
    Ok(log_dir.join("shu-net-keeper.log"))
}

/// 初始化 tracing 日志系统
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_log_path()?;

    // 创建带行数限制的 Writer，使用 Arc 包装以共享所有权
    let file_writer = WriterWrapper(Arc::new(LimitedLineWriter::new(log_path.clone())?));

    // 配置日志级别过滤器
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 初始化 tracing subscriber
    // 使用 move || file_writer.clone() 来为每次日志输出提供一个新的 writer 实例
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(move || file_writer.clone())
        .with_ansi(false) // 文件日志不使用颜色
        .with_target(false) // 不显示 target
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_span_events(FmtSpan::NONE)
        .init();

    tracing::info!("日志系统初始化成功");
    tracing::info!("日志文件路径: {}", log_path.display());
    tracing::info!("日志行数限制: {} 行", MAX_LOG_LINES);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_limited_line_writer() {
        let temp_path = PathBuf::from("test_log.txt");

        // 创建 writer
        let writer = LimitedLineWriter::new(temp_path.clone()).unwrap();

        // 写入 1100 行
        for i in 0..1100 {
            let _ = writeln!(&writer, "Line {}", i);
        }

        // 强制检查
        writer.check_and_truncate().unwrap();

        // 验证行数
        let count = count_lines(&temp_path).unwrap();
        assert!(count <= MAX_LOG_LINES, "行数应该不超过 {}", MAX_LOG_LINES);

        // 清理
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_truncate_to_last_n_lines() {
        let temp_path = PathBuf::from("test_truncate.txt");

        // 写入测试数据
        let mut file = File::create(&temp_path).unwrap();
        for i in 0..100 {
            writeln!(file, "Line {}", i).unwrap();
        }
        drop(file);

        // 截断到最后 10 行
        truncate_to_last_n_lines(&temp_path, 10).unwrap();

        // 验证
        let count = count_lines(&temp_path).unwrap();
        assert_eq!(count, 10);

        // 读取内容验证是最后 10 行
        let file = File::open(&temp_path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(lines[0], "Line 90");
        assert_eq!(lines[9], "Line 99");

        // 清理
        let _ = fs::remove_file(temp_path);
    }
}
