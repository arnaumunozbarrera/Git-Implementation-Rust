use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use lettre::Transport;
use lettre::message::Message;
use std::env;

#[derive(Clone, Debug)]
pub struct EmailAlert {
    pub recipient: String,
    pub contributor: String,
    pub commit_message: String,
    pub repository_name: String,
    pub branch: String,
    pub timestamp: String,
    pub top_files: Vec<(String, i64)>,
}

pub async fn send_push_alert(alert: EmailAlert) -> Result<(), String> {
    let smtp_host = env::var("SMTP_HOST").ok();
    let smtp_port = env::var("SMTP_PORT").ok();
    let smtp_user = env::var("SMTP_USER").ok();
    let smtp_password = env::var("SMTP_PASSWORD").ok();

    if smtp_host.is_none() || smtp_user.is_none() || smtp_password.is_none() {
        return Ok(());
    }

    let port: u16 = smtp_port
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);

    let html_body = format_html_email(&alert);
    let plain_body = format_plain_email(&alert);

    let email = Message::builder()
        .from(
            smtp_user
                .clone()
                .unwrap_or_default()
                .parse()
                .map_err(|_| "[ERROR] Invalid sender email".to_string())?
        )
        .to(
            alert.recipient
                .parse()
                .map_err(|_| "[ERROR] Invalid recipient email".to_string())?
        )
        .subject(format!("[Push] {} - {}", alert.repository_name, alert.commit_message.lines().next().unwrap_or("Unnamed commit")))
        .multipart(
            MultiPart::alternative()
                .singlepart(lettre::message::SinglePart::plain(plain_body))
                .singlepart(lettre::message::SinglePart::html(html_body))
        )
        .map_err(|e| format!("[ERROR] Failed to build email: {}", e))?;

    let credentials = Credentials::new(
        smtp_user.unwrap_or_default().into(),
        smtp_password.unwrap_or_default().into(),
    );

    let mailer = SmtpTransport::relay(smtp_host.unwrap_or_default().as_str())
        .map_err(|e| format!("[ERROR] SMTP relay error: {}", e))?
        .credentials(credentials)
        .port(port)
        .build();

    mailer
        .send(&email)
        .map_err(|e| format!("[ERROR] Failed to send email: {}", e))?;

    Ok(())
}

fn format_html_email(alert: &EmailAlert) -> String {
    let files_html = alert
        .top_files
        .iter()
        .map(|(file, changes)| {
            format!(
                "<tr><td style=\"padding: 8px; border-bottom: 1px solid #ddd;\">{}</td><td style=\"padding: 8px; border-bottom: 1px solid #ddd; text-align: right;\">{} changes</td></tr>",
                escape_html(file),
                changes
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f5f5f5; padding: 20px; }}
        .section {{ background: white; padding: 15px; margin-bottom: 15px; border-radius: 4px; }}
        .footer {{ color: #666; font-size: 12px; text-align: center; margin-top: 20px; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
        th {{ background: #f0f0f0; padding: 8px; text-align: left; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 New Push Alert</h1>
        </div>
        <div class="content">
            <div class="section">
                <h3>Push Details</h3>
                <p><strong>Repository:</strong> {}</p>
                <p><strong>Branch:</strong> {}</p>
                <p><strong>Contributor:</strong> {}</p>
                <p><strong>Date:</strong> {}</p>
            </div>
            <div class="section">
                <h3>Commit Message</h3>
                <p style="white-space: pre-wrap; color: #333;">{}</p>
            </div>
            <div class="section">
                <h3>Top 3 Files with Changes</h3>
                <table>
                    <thead>
                        <tr>
                            <th>File</th>
                            <th style="text-align: right;">Changes</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
        </div>
        <div class="footer">
            <p>This is an automated alert from your Git repository management system.</p>
        </div>
    </div>
</body>
</html>
"#,
        escape_html(&alert.repository_name),
        escape_html(&alert.branch),
        escape_html(&alert.contributor),
        escape_html(&alert.timestamp),
        escape_html(&alert.commit_message),
        files_html
    )
}

fn format_plain_email(alert: &EmailAlert) -> String {
    let files_text = alert
        .top_files
        .iter()
        .map(|(file, changes)| format!("  - {} ({} changes)", file, changes))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Push Alert

Repository: {}
Branch: {}
Contributor: {}
Date: {}

Commit Message:
{}

Top 3 Files with Changes:
{}

---
This is an automated alert from your Git repository management system.
"#,
        alert.repository_name,
        alert.branch,
        alert.contributor,
        alert.timestamp,
        alert.commit_message,
        files_text
    )
}

fn escape_html(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}
