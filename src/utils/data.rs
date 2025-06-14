pub static BODY: &str = r#"<!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8">
          <title>Password Reset</title>
        </head>
        <body style="margin:0; padding:0; font-family:Arial, sans-serif; background-color:#f4f4f4;">
          <table align="center" width="100%" cellpadding="0" cellspacing="0" style="padding: 40px 0;">
            <tr>
              <td align="center">
                <table width="600" cellpadding="0" cellspacing="0" style="background-color:#ffffff; border-radius:8px; box-shadow:0 2px 5px rgba(0,0,0,0.1); overflow:hidden;">
                  <tr>
                    <td style="padding: 20px; text-align:center; background-color:#007BFF; color:#ffffff;">
                      <h2 style="margin: 0;">Reset Your Password</h2>
                    </td>
                  </tr>
                  <tr>
                    <td style="padding: 30px 40px; color:#333333;">
                      <p style="font-size:16px; line-height:1.5;">
                        Hello,
                      </p>
                      <p style="font-size:16px; line-height:1.5;">
                        We received a request to reset your password. Click the button below to reset it. This link will expire in 30 minutes.
                      </p>
                      <div style="text-align:center; margin:30px 0;">
                        <a href="{{RESET_LINK}}" target="_blank"
                          style="display:inline-block; background-color:#007BFF; color:#ffffff; text-decoration:none; padding:12px 24px; border-radius:5px; font-size:16px;">
                          Reset Password
                        </a>
                      </div>
                      <p style="font-size:14px; color:#666666;">
                        If you didn't request this, you can safely ignore this email.
                      </p>
                    </td>
                  </tr>
                  <tr>
                    <td style="padding: 20px; text-align:center; font-size:12px; color:#999999; background-color:#f9f9f9;">
                      © {{YEAR}} Your Company. All rights reserved.
                    </td>
                  </tr>
                </table>
              </td>
            </tr>
          </table>
        </body>
        </html>"#;